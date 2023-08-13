import { createContext } from "solid-js";
import { useAppContext } from "@arena/sdk/app";
import { Store, StoreSetter, createStore } from "@arena/solid-store";
import { MutationQuery, createMutationQuery } from "@arena/uikit/solid";
import { uniqueId } from "@arena/sdk/utils/uniqueId";
import { jsonStreamToAsyncIterator } from "@arena/sdk/utils/stream";
import { Document } from "./types";

type SendMessageQuery = (message: string) => Promise<void>;
type Message = {
  id: string;
  sessionId: string;
  threadId: string | null;
  message: string;
  role: string;
  timestamp: Date;
  userId: string | null;
  metadata: {
    documents?: { documentId: string; score: number }[];
  } | null;
  /**
   * Set to true if this message is currently being streamed
   */
  streaming: boolean | undefined;
};

type State = {
  documents: null | Document[];
  activeSession: {
    id: string;
    messages: Message[];
  };
  isGeneratingResponse: boolean;
};

type ChatContext = {
  state: Store<State>;
  setState: StoreSetter<State>;
  sendNewMessage: MutationQuery<SendMessageQuery>;
};

const ChatContext = createContext<ChatContext>();

const ChatContextProvider = (props: any) => {
  const { router } = useAppContext();

  const [state, setState] = createStore<State>({
    documents: null,
    activeSession: {
      id: "1",
      messages: [],
    },
    isGeneratingResponse: false,
  });

  const listMessages = async (sessionId: string) => {
    return (await router.get(`/api/chat/${sessionId}/messages`)).data;
  };

  const fetchDocuments = async () => {
    return (await router.get<Document[]>("/api/documents")).data;
  };

  fetchDocuments().then((documents) => setState("documents", documents));
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
        `/api/chat/${sessionId}/send`,
        {
          id: messageId,
          message,
        },
        {
          responseType: "stream",
        }
      );
      readMessageStream(sessionId, res, setState);
    }
  );

  return (
    <ChatContext.Provider value={{ state, setState, sendNewMessage }}>
      {props.children}
    </ChatContext.Provider>
  );
};

const readMessageStream = async (
  sessionId: string,
  res: any,
  setState: StoreSetter<State>
) => {
  const stream = jsonStreamToAsyncIterator(res.body);
  let streamMsgIdx: number;
  let messageId: string | undefined;
  for await (const { json: chunk } of stream) {
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
            userId: null,
            sessionId,
            timestamp: chunk.timestamp,
            metadata: chunk.metadata,
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
          message: prev.message + chunk.text,
        };
      });
    }

    if (!messageId) {
      // Note(sagar): message id must be sent in the fist message
      throw new Error("Expected to received message id in the first chunk");
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

export { ChatContext, ChatContextProvider };
