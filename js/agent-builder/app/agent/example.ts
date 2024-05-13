import { Graph } from "./core/graph";
import { DefaultExecutor } from "./executor";

const executeGraph = async () => {
  const graph: Graph = {
    nodes: [
      {
        id: "chat-thread-1",
        type: "@core/chat-thread",
        config: {},
      },
      {
        id: "chat-completion-1",
        type: "@core/chat-completion",
        config: {
          systemPrompt: "You are an AI assistant called Atlas",
          temperature: 0.7,
          stream: true,
        },
      },
    ],
    edges: [
      {
        id: "edge-1",
        from: {
          node: "input-1",
          outputKey: "threadId",
        },
        to: {
          node: "chat-thread-1",
          inputKey: "threadId",
        },
      },
      {
        id: "edge-2",
        from: {
          node: "input-1",
          outputKey: "query",
        },
        to: {
          node: "chat-completion-1",
          inputKey: "query",
        },
      },
      {
        id: "edge-3",
        from: {
          node: "chat-thread-1",
          outputKey: "history",
        },
        to: {
          node: "chat-completion-1",
          inputKey: "chatHistory",
        },
      },
    ],
  };

  const executor = new DefaultExecutor();
  executor.compileGraph(graph);

  executor.subscribe((event) => {
    console.log("EVENT =", event);
  });
  executor.trigger(
    { id: "input-1" },
    {
      // query: "Who are you?",
      query: "Who am I?",
      threadId: null,
    }
  );
};

executeGraph();
