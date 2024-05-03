export type ChatThread = {
  id: string;
  title: string;
  /**
   * If a thread is blocked, this field is set to whoever blocked the
   * thread. For example, the thread will be blocked when a workflow
   * is running
   */
  blockedBy: string | null;
  metadata: {
    model: {
      id: string;
      name: string;
    };
    // chat profile
    profile?: {
      id: string;
      name: string;
    };
    activeWorkflow?: {
      id: string;
    };
    agent?: {
      id: string;
      name: string;
      layout: "show-agentpanel"[];
    };
  };
  createdAt: Date;
};

export type Message = {
  id: string;
  threadId: string | null;
  parentId: string | null;
  message: {
    function_call?: string;
    content?: string | MessageContent[];
  };
  role: string;
  createdAt: Date;
  userId: string | null;
  metadata: {
    error?: string;
    documents?: { documentId: string; score: number }[];
  } | null;
};

type MessageContent =
  | {
      type: "text";
      text: string;
    }
  | {
      type: "image_url";
      // `data:image/png;base64,...`
      image_url: string;
    };
