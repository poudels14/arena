import { createContext } from "solid-js";
import { useAppContext } from "@arena/sdk/app";
import { Store, StoreSetter, createStore } from "@arena/solid-store";
import { MutationQuery, createMutationQuery } from "@arena/uikit/solid";
import { uniqueId } from "@arena/sdk/utils/uniqueId";

type SendMessageQuery = (message: string) => Promise<void>;
type Message = {
  id: string;
  sessionId: string;
  threadId: string | null;
  message: string;
  role: string;
  timestamp: Date;
  userId: string | null;
  /**
   * Set to true if this message is currently being streamed
   */
  streaming: boolean | undefined;
};

type State = {
  activeSession: {
    id: string;
    messages: Message[];
  };
  isGeneratingResponse: boolean;
};

type ChatContext = {
  state: Store<State>;
  sendNewMessage: MutationQuery<SendMessageQuery>;
};

const ChatContext = createContext<ChatContext>();

const ChatContextProvider = (props: any) => {
  const { router } = useAppContext();

  const [state, setState] = createStore<State>({
    activeSession: {
      id: "1",
      messages: [],
    },
    isGeneratingResponse: false,
  });

  const listMessages = async (sessionId: string) => {
    return (await router.get(`/chat/${sessionId}/history`)).data;
  };

  listMessages("1").then((messages: any[]) => {
    const m: Message[] = messages.map((m) => {
      return {
        ...m,
        timestamp: new Date(m.timestamp),
      };
    });
    setState("activeSession", "messages", m as any);
  });

  const sendNewMessage = createMutationQuery<SendMessageQuery>(
    async (message) => {
      const messageId = uniqueId();
      const sessionId = state.activeSession.id();
      setState("isGeneratingResponse", true);
      setState("activeSession", "messages", (prev: any) => {
        return [
          ...prev,
          {
            id: messageId,
            message,
            role: "user",
            sessionId,
            timestamp: new Date().getTime(),
            userId: null,
          },
        ];
      });

      const res = await router.post(
        `/chat/${sessionId}/send`,
        {
          message: { id: messageId, message },
        },
        {
          responseType: "stream",
        }
      );
      readMessageStream(sessionId, res, setState);
    }
  );

  return (
    <ChatContext.Provider value={{ state, sendNewMessage }}>
      {props.children}
    </ChatContext.Provider>
  );
};

const readMessageStream = async (
  sessionId: string,
  res: any,
  setState: StoreSetter<State>
) => {
  const stream = responseStreamGenerator(res.body);
  let streamMsgIdx: number;
  let messageId: string;
  for await (const chunk of stream) {
    if (chunk.id) {
      messageId = chunk.id;
      setState("activeSession", "messages", (prev: any[]) => {
        if (prev.find((m) => m.id == chunk.id)) {
          throw new Error("Duplicate messages found");
        }
        streamMsgIdx = prev.length;
        return [
          ...prev,
          {
            id: messageId,
            message: "",
            role: "ai",
            sessionId,
            timestamp: chunk.timestamp,
            userId: null,
            streaming: true,
          },
        ];
      });
    }
    if (chunk.text) {
      setState("activeSession", "messages", streamMsgIdx!, (prev: any) => {
        if (prev.id !== messageId) {
          // TODO(sagar): instead of throwing error here,
          // find the new index
          throw new Error("Invalid message index");
        }
        return {
          ...prev,
          message: prev.message + chunk.text + " ",
        };
      });
    }
  }
  setState("activeSession", "messages", streamMsgIdx!, (prev: any) => {
    if (prev.id !== messageId) {
      // TODO(sagar): instead of throwing error here,
      // find the new index
      throw new Error("Invalid message index");
    }
    return {
      ...prev,
      streaming: undefined,
    };
  });
  setState("isGeneratingResponse", false);
};

async function* responseStreamGenerator(body: Response["body"]) {
  const reader = body?.getReader();
  let data = await reader?.read();
  var enc = new TextDecoder("utf-8");
  while (data && !data.done) {
    const text = enc.decode(data?.value);
    if (text.startsWith("data:")) {
      yield JSON.parse(text.substring(5));
    }
    data = await reader?.read();
  }
}

export { ChatContext, ChatContextProvider };
