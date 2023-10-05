import {
  createSignal,
  createContext,
  createComputed,
  createEffect,
  batch,
} from "solid-js";
import { useNavigate } from "@solidjs/router";
import { useAppContext } from "@arena/sdk/app";
import { Store, StoreSetter, createStore } from "@arena/solid-store";
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
      setState("threads", threadId, "blocked", true);
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
        await readMessageStream(channelId, threadId, res, setState);
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
      setState("threads", threadId, "blocked", false);
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
          threads.forEach((t: Chat.Thread) => {
            newThreads[t.id] = t;
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
        fetchThread(channelId, threadId).then((thread: Chat.Thread) => {
          // TODO(sagar): reconcile
          const { messages, ...rest } = thread;
          setState("threads", threadId, rest);
          setState("threads", threadId, "messages", (prev = []) => {
            const newMessages = messages.filter(
              (m) => !prev.find((p) => p.id == m.id)
            );
            return [...prev, ...newMessages].sort(
              (a, b) => a.timestamp - b.timestamp
            );
          });
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
  setState: StoreSetter<ChatState>
) => {
  const stream = jsonStreamToAsyncIterator(res.body);
  let messageId: string | undefined;

  const content = createSignal("");
  for await (const { json: chunk } of stream) {
    if (chunk.id) {
      messageId = chunk.id;
      setState("threads", threadId, "messages", (prev) => {
        const messages = prev.filter((p) => p.id !== messageId);
        messages.push({
          id: messageId!,
          threadId,
          message: {
            role: "ai",
            content: content[0],
          },
          role: "ai",
          userId: null,
          channelId,
          timestamp: chunk.timestamp,
          metadata: chunk.metadata,
          streaming: true,
        });
        return messages;
      });
    }

    if (chunk.delta?.content) {
      content[1](content[0]() + chunk.delta.content);
    }

    if (!messageId) {
      // Note(sagar): message id must be sent in the fist message
      throw new Error("Expected to received message id in the first chunk");
    }
  }
  setState("threads", threadId, "messages", (prev) => {
    return prev.map((m) => {
      if (m.id == messageId) {
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
