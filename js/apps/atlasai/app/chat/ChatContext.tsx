import {
  createContext,
  Accessor,
  createComputed,
  createMemo,
  createSignal,
  batch,
} from "solid-js";
import { Store, UNDEFINED_PROXY, createStore } from "@portal/solid-store";
import {
  MutationQuery,
  createMutationQuery,
  createQuery,
} from "@portal/solid-query";
import cleanSet from "clean-set";
import { Chat } from "../types";
import {
  SharedWorkspaceContext,
  useSharedWorkspaceContext,
} from "@portal/workspace-sdk";

export type ChatState = {
  activeThreadId: Accessor<string | undefined>;
  threadsById: Store<Record<string, ChatThread>>;
};

type ChatThread = Omit<Chat.Thread, "messages"> & {
  messages: Record<string, Chat.Message>;
};

type ChatQueryContext = NonNullable<
  ReturnType<SharedWorkspaceContext["getChatContext"]>
>;

type ChatContext = {
  state: ChatState;
  sortedMessageIds: () => string[];
  aiMessageIdsByParentId: () => Record<string, string[]>;
  selectedMessageVersionByParentId: Accessor<Record<string, string>>;
  selectMessageVersion: (parentId: string, id: string) => void;
  getActiveChatThread: () => Store<ChatThread>;
  getChatProfiles: () => Chat.Profile[];
  refreshThreadsById: () => void;
  sendNewMessage: MutationQuery<
    {
      id: string;
      threadId: string;
      message: { content: string };
      context: ChatQueryContext;
      isNewThread: boolean;
      regenerate: boolean;
      idFilter?: string[];
    },
    any
  >;
  regenerateMessage: (options: { id: string }) => void;
};

const ChatContext = createContext<ChatContext>();

const ChatContextProvider = (props: {
  activeThreadId: string | undefined;
  onThreadReady?: (threadId: string) => void;
  children: any;
}) => {
  const { activeWorkspace, getChatConfig } = useSharedWorkspaceContext();
  const [chatThreadsById, setChatThreadsById] = createStore<
    Record<string, ChatThread>
  >({});
  // stores parentId => selected message version id
  const [selectedMessageVersion, setSelectedMessageVersion] = createSignal<
    Record<string, string>
  >({});

  const listThreadsRoute = createQuery<any[]>(() => {
    return "/chat/threads";
  }, {});

  createComputed(() => {
    const threads = listThreadsRoute.data();
    batch(() => {
      threads?.forEach((thread) => {
        setChatThreadsById(thread.id, (prev) => {
          return { ...(prev || {}), ...thread };
        });
      });
    });
  });

  const activeThreadRoute = createQuery<Chat.Thread>(() => {
    if (!props.activeThreadId) {
      return null;
    }
    return `/chat/threads/${props.activeThreadId}`;
  }, {});

  const chatProfiles = createQuery<Chat.Profile[]>(() => {
    return `/chat/profiles`;
  }, {});

  // refresh chat message when props id change
  createComputed(() => {
    void props.activeThreadId;
    if (props.activeThreadId) {
      // set default messages for thread if it's not already set
      setChatThreadsById(props.activeThreadId, (prev) => {
        return (
          prev || {
            blockedBy: null,
            messages: {},
            createdAt: new Date(),
          }
        );
      });
    }
    activeThreadRoute.refresh();
  });

  createComputed(() => {
    const data = activeThreadRoute.data()!;
    if (!data) {
      return;
    }
    const messages = data.messages || [];
    batch(() => {
      setChatThreadsById(data.id, "id", data.id!);
      setChatThreadsById(data.id, "title", data.title!);
      setChatThreadsById(data.id, "blockedBy", data.blockedBy || null);
      setChatThreadsById(data.id, "metadata", data.metadata!);
      setChatThreadsById(data.id, "messages", (prev) => {
        return messages.reduce(
          (agg, message) => {
            agg[message.id] = {
              ...message,
              createdAt: new Date(message.createdAt).toISOString(),
            };
            return agg;
          },
          { ...prev } as Record<string, Chat.Message>
        );
      });
    });
  });

  const getActiveChatThread = () => {
    if (!props.activeThreadId) {
      return UNDEFINED_PROXY as Store<ChatThread>;
    }
    return chatThreadsById[props.activeThreadId!] as Store<ChatThread>;
  };

  const messagesByParentId = createMemo(() => {
    const messages = Object.values(getActiveChatThread().messages() || []);
    messages.sort(
      (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
    );
    const messagesByParentId: Record<string, Chat.Message[]> = {};
    messages.forEach((message) => {
      if (!messagesByParentId[message.parentId!]) {
        messagesByParentId[message.parentId!] = [];
      }
      messagesByParentId[message.parentId!].push(message);
    });
    return messagesByParentId;
  });

  const aiMessageIdsByParentId = createMemo(() => {
    return Object.fromEntries(
      Object.entries(messagesByParentId()).map(([parentId, messages]) => [
        parentId,
        messages.filter((m) => m.role == "ai").map((m) => m.id),
      ])
    );
  });

  const sortedMessageIds = createMemo(() => {
    const versionByParentId = selectedMessageVersion();
    const childrenByParentId = messagesByParentId();
    const chain: string[] = [];
    let parentId: string = null!;
    let children: Chat.Message[] = childrenByParentId[parentId!];
    while (children) {
      const selectedChildId =
        versionByParentId[parentId] || children[children.length - 1].id;
      children.forEach((child) => {
        if (child.role == "system" || child.id == selectedChildId) {
          chain.push(child.id);
        }
      });
      parentId = selectedChildId;
      children = childrenByParentId[selectedChildId!];
    }
    return chain;
  });

  const sendNewMessage = createMutationQuery<{
    id: string;
    threadId: string;
    message: { content: string };
    regenerate: boolean;
    context: ChatQueryContext;
    isNewThread: boolean;
    idFilter?: string[];
  }>((input) => {
    const chatConfig = getChatConfig();
    const allMessages = getActiveChatThread().messages();
    const lastAIMessageId = sortedMessageIds().findLast(
      (id) => allMessages[id].role == "ai"
    );
    const activeThread = getActiveChatThread();
    const selectedMessages = sortedMessageIds();
    // If it's a new thread, navigate to that thread first
    return {
      url: `/chat/threads/${input.threadId}/send`,
      request: {
        body: {
          id: input.id,
          model: {
            id:
              activeThread.metadata.model.id() ||
              chatConfig.model ||
              activeWorkspace.models().find((m) => !m.disabled)?.id,
          },
          message: input.message,
          parentId: lastAIMessageId || null,
          idFilter: input.idFilter || selectedMessages,
          regenerate: input.regenerate,
          selectedChatProfileId: chatConfig.selectedProfileId,
          context: input.context,
        },
        headers: {
          "content-type": "text/event-stream",
        },
      },
    };
  });

  createComputed(() => {
    const input = sendNewMessage.input()!;
    if (sendNewMessage.status && input) {
      props.onThreadReady?.(input.threadId);
    }
  });

  createComputed(() => {
    const threadId = props.activeThreadId!;
    if (sendNewMessage.status != 200) return;
    sendNewMessage.stream((data) => {
      if (data.ops) {
        data.ops.forEach((op: any) => {
          const [pathPrefix, ...path] = op.path;
          if (pathPrefix == "messages") {
            setChatThreadsById(threadId, "messages", (prev) => {
              const value = op.value;
              let messages = prev;
              if (op.op == "replace") {
                messages = cleanSet(messages, path, value);
              } else if (op.op == "add") {
                messages = cleanSet(messages, path, (prev) => {
                  if (typeof value == "string") {
                    return (prev || "") + value;
                  } else if (Array.isArray(prev)) {
                    return [...(prev || []), value];
                  } else {
                    return value;
                  }
                });
              }
              return messages;
            });
          } else if (pathPrefix == "threads") {
            setChatThreadsById((prev) => {
              let threadsById = prev;
              if (op.op == "replace") {
                threadsById = cleanSet(threadsById, path, op.value);
              }
              return threadsById;
            });
          }
        });
      }
    });
  });

  // regenerate basicaly re-sends parent message sent by the user
  const regenerateMessage = async (options: { id: string }) => {
    const allMessages = getActiveChatThread().messages();
    const message = allMessages[options.id];
    const parentMessage = allMessages[message.parentId!];

    const messageIds = sortedMessageIds();
    const idFilter = messageIds.slice(
      0,
      messageIds.findIndex((id) => id == parentMessage.id)
    );

    await sendNewMessage.mutate({
      id: parentMessage.id,
      threadId: parentMessage.threadId!,
      message: {
        content: parentMessage.message.content!,
      },
      idFilter,
      regenerate: true,
      context: parentMessage.metadata?.context || [],
      isNewThread: false,
    });
  };

  return (
    <ChatContext.Provider
      value={{
        state: {
          activeThreadId() {
            return props.activeThreadId;
          },
          get threadsById() {
            void chatThreadsById();
            return chatThreadsById;
          },
        },
        refreshThreadsById() {
          listThreadsRoute.refresh();
        },
        getActiveChatThread,
        getChatProfiles() {
          return chatProfiles.data() || [];
        },
        sortedMessageIds,
        aiMessageIdsByParentId,
        selectedMessageVersionByParentId: selectedMessageVersion,
        selectMessageVersion(parentId, id) {
          setSelectedMessageVersion((prev) => {
            return {
              ...prev,
              [parentId]: id,
            };
          });
        },
        sendNewMessage,
        regenerateMessage,
      }}
    >
      {props.children}
    </ChatContext.Provider>
  );
};

export { ChatContext, ChatContextProvider };
export type { ChatQueryContext };
