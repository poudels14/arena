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
    /**
     * If a thread is blocked, this field is set.
     * The thread will be blocked when AI is generating response or
     * a workflow is running
     */
    blockedBy?: string | null;
    metadata: {
      ai: {
        model: string;
      };
    };
    timestamp: number;

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
      function_call?: string;
      content?: string;
    };
    role: string;
    timestamp: number;
    userId: string | null;
    metadata: {
      documents?: { documentId: string; score: number }[];
      function?: {
        type: "workflow";
        id: string;
      };
    } | null;
    /**
     * Set to true if this message is currently being streamed
     */
    streaming: boolean | undefined;
  };
}
