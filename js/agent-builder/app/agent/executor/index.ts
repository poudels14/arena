import { uniqueId } from "@portal/cortex/utils/uniqueId";
import { AgentNode, ZodObjectSchema } from "../core/node";
import { EventStream } from "../core/stream";
import { Graph } from "../core/graph";
import { ActorNode } from "./ActorNode";
import { ChatThread } from "../nodes/ChatThread";
import { ChatCompletion } from "../nodes/ChatCompletion";

class DefaultExecutor {
  stream: EventStream<any>;
  agentNodesByName: Record<string, any>;

  constructor() {
    this.stream = new EventStream({
      run: {
        id: uniqueId(),
      },
    });

    this.agentNodesByName = Object.fromEntries(
      [ChatThread, ChatCompletion].map((node) => [
        new node().metadata.name,
        node,
      ])
    );
  }

  addAgentNode<
    C extends ZodObjectSchema,
    I extends ZodObjectSchema,
    O extends ZodObjectSchema
  >(node: AgentNode<C, I, O>) {
    throw new Error("not implemented");
  }

  compileGraph(graph: Graph) {
    const actors = graph.nodes.map((node) => {
      const nodeEdges = graph.edges.filter(
        (edge) => edge.to.node == node.id || edge.from.node == node.id
      );
      const Node = this.agentNodesByName[node.type];
      return new ActorNode(node.id, new Node(), node.config, nodeEdges);
    });

    actors.forEach((actor) => {
      actor.subscribe(this.stream);
    });
  }

  trigger(node: { id: string }, input: any) {
    this.stream.sendOutput(node, input);
  }

  subscribe(callback: (event: any) => void) {
    this.stream.subscribe((event) => {
      callback(event);
    });
  }
}

export { DefaultExecutor };
