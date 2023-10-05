import { createContext, createComputed, createEffect, batch } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { useAppContext } from "@arena/sdk/app";
import { $RAW, Store, StoreSetter, createStore } from "@arena/solid-store";
import { MutationQuery, createMutationQuery } from "@arena/uikit/solid";
import { uniqueId } from "@arena/sdk/utils/uniqueId";
import { jsonStreamToAsyncIterator } from "@arena/sdk/utils/stream";
import { Document, Chat } from "../types";

export type ChatState = {
  activeChannelId: string | null;
  activeThreadId: string | null;
  channels: Chat.Channel[];
  activeChannel: Chat.Channel | null;
  threads: Record<string, Chat.Thread>;
  documents: null | Document[];
  errors: Chat.Error[];
};

type ChatContext = {
  state: Store<ChatState>;
  setState: StoreSetter<ChatState>;
  setChatChannel: (channelId: string, threadId?: string) => void;
  sendNewMessage: MutationQuery<Chat.SendMessageQuery>;
};

const ChatContext = createContext<ChatContext>();

const ChatContextProvider = (props: any) => {
  const navigate = useNavigate();
  const { router } = useAppContext();

  const [state, setState] = createStore<ChatState>({
    activeChannelId: "default",
    activeThreadId: null,
    activeChannel: null,
    channels: [],
    threads: {},
    documents: null,
    errors: [],
  });

  const setChatChannel = (channelId: string, threadId?: string) => {
    batch(() => {
      setState("activeChannelId", channelId || "default");
      setState("activeThreadId", threadId || null);
    });
  };

  const sendNewMessage = createMutationQuery<Chat.SendMessageQuery>(
    async (message) => {
      const messageId = uniqueId();
      const channelId = state.activeChannelId() || "default";
      const activeThreadId = state.activeThreadId();
      const threadId = activeThreadId || messageId;

      // If it's a new thread, set the initial state
      setState("threads", threadId, "blockedBy", "UI");
      setState("threads", threadId, "messages", (prev = []) => {
        return [
          ...prev,
          {
            id: messageId,
            message: {
              role: "user",
              content: message,
            },
            threadId,
            metadata: {},
            streaming: false,
            role: "user",
            channelId,
            timestamp: new Date().getTime(),
            userId: null,
          },
        ];
      });
      if (!activeThreadId) {
        navigate(`/chat/${channelId}/t/${threadId}`);
      }

      const res = await router.post(
        `/api/chat/${channelId}/send`,
        {
          id: messageId,
          thread: {
            id: threadId,
            ai: {},
          },
          message,
        },
        {
          responseType: "stream",
        }
      );

      if (res.status == 200) {
        await readMessageStream(channelId, threadId, res, state, setState);
      } else {
        setState("errors", (prev) => {
          return [
            ...prev,
            {
              channelId,
              threadId,
              message: "Something went wrong. Please try again.",
            },
          ];
        });
      }
      setState("threads", threadId, "blockedBy", (prev) => {
        return prev == "UI" ? null : prev;
      });
    }
  );

  const fetchChannel = async (channelId: string) => {
    return (await router.get(`/api/chat/channels/${channelId}`)).data;
  };

  const fetchThread = async (channelId: string, threadId: string) => {
    return (await router.get(`/api/chat/${channelId}/threads/${threadId}`))
      .data;
  };

  const fetchDocuments = async () => {
    return (await router.get<Document[]>("/api/documents")).data;
  };

  fetchDocuments().then((documents) => setState("documents", documents));
  router
    .get("/api/chat/channels")
    .then((res) => res.data)
    .then((channels) => {
      setState("channels", channels);
    });

  createComputed(() => {
    const activeChannelId = state.activeChannelId() || "default";
    fetchChannel(activeChannelId).then(({ threads, messages, ...channel }) => {
      batch(() => {
        setState("activeChannel", channel);
        setState("channels", (prev: any[]) => {
          const channels = prev.filter((c) => c.id !== activeChannelId);
          channels.push(channel);
          return channels;
        });
        setState("threads", (prev) => {
          const newThreads = { ...prev };
          // TODO(sagar): reconcile
          threads.forEach((t: Chat.Thread) => {
            if (!newThreads[t.id]) {
              newThreads[t.id] = t;
            }
          });
          return newThreads;
        });
      });
    });
  });

  createEffect<any[]>((prev) => {
    const channelId = state.activeChannelId() || "default"!;
    const threadId = state.activeThreadId()!;
    if ((channelId && channelId != prev[0]) || threadId != prev[1]) {
      if (threadId) {
        fetchThread(channelId, threadId)
          .then((thread: Chat.Thread) => {
            // TODO(sagar): reconcile
            const { messages, ...rest } = thread;
            setState("threads", threadId, rest);
            setState("threads", threadId, "messages", (prev = []) => {
              // TODO(sagar): reconcile existing messages
              const newMessages = messages.filter(
                (m) => !prev.find((p) => p.id == m.id)
              );
              return [...prev, ...newMessages].sort(
                (a, b) => a.timestamp - b.timestamp
              );
            });
          })
          .catch((e) => {
            if (e.status == 404) {
              navigate(`/chat/${channelId}`);
            }
          });
      }
    }
    return [channelId, threadId];
  }, []);

  return (
    <ChatContext.Provider
      value={{ state, setState, setChatChannel, sendNewMessage }}
    >
      {props.children}
    </ChatContext.Provider>
  );
};

const readMessageStream = async (
  channelId: string,
  threadId: string,
  res: any,
  state: Store<ChatState>,
  setState: StoreSetter<ChatState>
) => {
  const stream = jsonStreamToAsyncIterator(res.body);
  let streamingMessageId: string | undefined;
  let messageIdx: number | undefined;

  for await (const { json: chunk } of stream) {
    const { message, thread } = chunk;
    if (message) {
      if (messageIdx == undefined && message.id) {
        streamingMessageId = message.id;

        setState("threads", threadId, "messages", (prev) => {
          messageIdx = prev.length;
          const messages = [
            ...prev,
            {
              id: message.id!,
              threadId,
              message: {
                role: "assistant",
                content: "",
              },
              role: "ai",
              userId: null,
              channelId,
              timestamp: message.timestamp,
              metadata: message.metadata,
              streaming: true,
            },
          ];
          messages.sort((a, b) => a.timestamp - b.timestamp);
          return messages;
        });
      } else {
        const allMessages = state.threads[threadId].messages[$RAW];
        if (allMessages[messageIdx!].id != message.id) {
          messageIdx = allMessages.findIndex((m) => m.id == message.id);
          if (messageIdx == undefined) {
            throw new Error("Something unexpected happened");
          }
        }

        if (message.delta?.content) {
          setState(
            "threads",
            threadId,
            "messages",
            messageIdx!,
            "message",
            "content",
            (prev) => prev + message.delta.content
          );
        }
        if (message.metadata) {
          setState(
            "threads",
            threadId,
            "messages",
            messageIdx!,
            "metadata",
            message.metadata
          );
        }
      }
    }
    if (thread) {
      setState("threads", thread.id, (prev) => {
        return {
          ...prev,
          ...thread,
        };
      });
    }
  }
  setState("threads", threadId, "messages", (prev) => {
    return prev.map((m) => {
      if (m.id == streamingMessageId) {
        return {
          ...m,
          streaming: undefined,
        };
      }
      return m;
    });
  });
};

export { ChatContext, ChatContextProvider };
