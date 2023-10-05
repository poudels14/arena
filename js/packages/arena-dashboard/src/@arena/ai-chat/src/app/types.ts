import { Accessor } from "solid-js";

export type Document = {
  id: string;
  name: string;
  isNew?: boolean;
};

export namespace Chat {
  export type Channel = {
    id: string;
    name: string;
    metadata: {
      enableAI?: boolean;
    };
  };

  export type Thread = {
    id: string;
    channelId: string;
    title: string;
    metadata: {
      ai: {
        model: string;
      };
    };
    timestamp: number;

    /**
     * If a thread is blocked, user can't send the message.
     * The thread should be blocked when AI is generating response
     */
    blocked: boolean;
    messages: Message[];
  };

  export type Error = {
    message: string;
    channelId: string;
    threadId: string | null;
  };

  export type SendMessageQuery = (message: string) => Promise<void>;
  export type Message = {
    id: string;
    channelId: string;
    threadId: string | null;
    message: {
      role: string;
      content: string | Accessor<string>;
    };
    role: string;
    timestamp: number;
    userId: string | null;
    metadata: {
      documents?: { documentId: string; score: number }[];
    } | null;
    /**
     * Set to true if this message is currently being streamed
     */
    streaming: boolean | undefined;
  };
}
