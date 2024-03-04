import { createContext, Accessor, createComputed } from "solid-js";
import { Store, UNDEFINED_PROXY, createStore } from "@portal/solid-store";
import {
  MutationQuery,
  createMutationQuery,
  createQuery,
} from "@portal/solid-query";
import cleanSet from "clean-set";
import { Chat } from "../types";
import { SharedWorkspaceContext } from "@portal/workspace-sdk";

export type ChatState = {
  activeThreadId: Accessor<string | undefined>;
  threadsById: Store<Record<string, Chat.Thread>>;
};

type ChatThread = {
  blockedBy: string | null;
  messages: Record<string, Chat.Message>;
};

type ChatQueryContext = NonNullable<
  ReturnType<SharedWorkspaceContext["getChatContext"]>
>;

type ChatContext = {
  state: ChatState;
  getActiveChatThread: () => Store<ChatThread>;
  sendNewMessage: MutationQuery<
    {
      id: string;
      threadId: string;
      message: { content: string };
      context: ChatQueryContext;
      isNewThread: boolean;
    },
    any
  >;
};

const ChatContext = createContext<ChatContext>();

const ChatContextProvider = (props: {
  activeThreadId: string | undefined;
  onThreadReady?: (threadId: string) => void;
  children: any;
}) => {
  const threadsRoute = createQuery<any[]>(
    () => {
      return "/chat/threads";
    },
    {},
    {
      derive: {
        threadsById: (query, prev: any) => {
          const threads = query.data();
          if (threads) {
            const theadsById = { ...prev };
            // TODO(sagar): reconcile
            threads.forEach((thread: Chat.Thread) => {
              if (!theadsById[thread.id]) {
                theadsById[thread.id] = thread;
              }
            });
            return theadsById;
          }
        },
      },
    }
  );

  const [chatThreadsById, setChatThreadsById] = createStore<
    Record<string, ChatThread>
  >({});

  const activeThreadRoute = createQuery<Chat.Thread>(() => {
    if (!props.activeThreadId) {
      return null;
    }
    return `/chat/threads/${props.activeThreadId}`;
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
    setChatThreadsById(data.id, "blockedBy", data.blockedBy || null);
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

  const sendNewMessage = createMutationQuery<{
    id: string;
    threadId: string;
    message: { content: string };
    context: ChatQueryContext;
    isNewThread: boolean;
  }>((input) => {
    // If it's a new thread, navigate to that thread first
    return {
      url: `/chat/threads/${input.threadId}/send`,
      request: {
        body: {
          id: input.id,
          message: input.message,
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
            threadsRoute.setState<any>("threadsById", (prev) => {
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

  return (
    <ChatContext.Provider
      value={{
        state: {
          activeThreadId() {
            return props.activeThreadId;
          },
          get threadsById() {
            return threadsRoute.state<Record<string, Chat.Thread>>(
              "threadsById"
            );
          },
        },
        getActiveChatThread() {
          if (!props.activeThreadId) {
            return UNDEFINED_PROXY;
          }
          return chatThreadsById[props.activeThreadId!];
        },
        sendNewMessage,
      }}
    >
      {props.children}
    </ChatContext.Provider>
  );
};

export { ChatContext, ChatContextProvider };
export type { ChatQueryContext };
