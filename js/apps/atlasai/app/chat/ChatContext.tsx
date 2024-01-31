import { createContext, Accessor } from "solid-js";
import { Store } from "@portal/solid-store";
import { createQuery } from "@portal/solid-query";
import { Chat } from "../types";

export type ChatState = {
  activeThreadId: Accessor<string | undefined>;
  threadsById: Store<Record<string, Chat.Thread>>;
};

type ChatContext = {
  state: ChatState;
};

const ChatContext = createContext<ChatContext>();

const ChatContextProvider = (props: {
  activeThreadId: string | undefined;
  children: any;
}) => {
  const threadsRoute = createQuery<any[]>(
    () => "/chat/threads",
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
      }}
    >
      {props.children}
    </ChatContext.Provider>
  );
};

export { ChatContext, ChatContextProvider };
