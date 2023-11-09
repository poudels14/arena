export type ChatChannel = {
  id: string;
  name: string;
  metadata: {
    enableAI: boolean;
  };
};

export type ChatThread = {
  id: string;
  title: string;
  /**
   * If a thread is blocked, this field is set to whoever blocked the
   * thread. For example, the thread will be blocked when a workflow
   * is running
   */
  blockedBy?: string;
  metadata: {
    ai: {
      model: string;
    };
    activeWorkflow?: {
      id: string;
    };
  };
};
