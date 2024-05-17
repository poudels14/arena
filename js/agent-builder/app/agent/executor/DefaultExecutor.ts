import { uniqueId } from "@portal/cortex/utils/uniqueId";

import { AgentNode, ZodObjectSchema } from "../core/node";
import { EventStream } from "../core/stream";
import { Graph } from "../core/graph";
import { ActorNode } from "./ActorNode";
import { ChatThread } from "../nodes/ChatThread";
import { ChatCompletion } from "../nodes/ChatCompletion";
import { AgentInput } from "../nodes/AgentInput";
import { User } from "../nodes/User";

class DefaultExecutorBuilder {
  #agentNodesByName: Record<string, any>;

  constructor() {
    this.#agentNodesByName = Object.fromEntries(
      [AgentInput, User, ChatThread, ChatCompletion].map((node) => [
        new node().metadata.id,
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
      const Node = this.#agentNodesByName[node.type];
      console.log("node =", node);
      if (!Node) {
        throw new Error(`Agent node of type [${node.type}] not found`);
      }
      return new ActorNode(node.id, new Node(), node.config, nodeEdges);
    });

    return new DefaultExecutor(actors);
  }
}

class DefaultExecutor {
  #actors: ActorNode<any, ZodObjectSchema, ZodObjectSchema>[];
  constructor(actors: ActorNode<any, ZodObjectSchema, ZodObjectSchema>[]) {
    this.#actors = actors;
  }

  newExecutionStream() {
    return new ExecutionStream(this.#actors);
  }
}

class ExecutionStream {
  #stream: EventStream<any>;

  constructor(actors: ActorNode<any, ZodObjectSchema, ZodObjectSchema>[]) {
    this.#stream = new EventStream({
      run: {
        id: uniqueId(),
      },
    });

    actors.forEach((actor) => {
      actor.subscribe(this.#stream);
    });
  }

  trigger(node: { id: string }, input: any) {
    this.#stream.sendOutput(node, input);
  }

  subscribe(callback: (event: any) => void) {
    this.#stream.subscribe((event) => {
      callback(event);
    });
  }
}

export { DefaultExecutorBuilder };
