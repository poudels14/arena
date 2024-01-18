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
      activeWorkflow?: {
        id: string;
      };
    };
    createdAt: number;

    messages: Message[];
  };

  export type Error = {
    message: string;
    threadId: string | null;
  };

  export type Message = {
    id: string;
    threadId: string | null;
    message: {
      content?: string;
      tool_calls: {
        id: string;
        type: string;
        function: {
          name: string;
          arguments: any;
        };
      }[];
    };
    role: string;
    createdAt: string;
    userId: string | null;
    metadata: {
      documents?: { documentId: string; score: number }[];
      workflow?: {
        id: string;
      };
    } | null;
    /**
     * Set to true if this message is currently being streamed
     */
    streaming: boolean | undefined;
  };

  export type TaskExecution = {
    id: string;
    taskId: string;
    threadId: string;
    messageId: string;
    status: "STARTED" | string;
    metadata: any;
    state: any;
    startedAt: string;
  };
}
