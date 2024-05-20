import { uniqueId } from "@portal/cortex/utils/uniqueId";
import {
  ZodObjectSchema,
  Graph,
  EventStream,
  AgentNode,
} from "@portal/cortex/agent";

import { ActorNode } from "./ActorNode";
import { ChatCompletion } from "../nodes/ChatCompletion";
import { AgentTools } from "../nodes/AgentTools";
import { AgentToolExecutor } from "../nodes/AgentToolExecutor";
import { AgentInput } from "../nodes/AgentInput";
import { User } from "../nodes/User";

class DefaultExecutorBuilder {
  #agentNodesByName: Record<string, any>;

  constructor() {
    this.#agentNodesByName = Object.fromEntries(
      [AgentInput, User, ChatCompletion, AgentTools, AgentToolExecutor].map(
        (node) => [new node().metadata.id, node]
      )
    );
  }

  addAgentNode<
    C extends ZodObjectSchema,
    I extends ZodObjectSchema,
    O extends ZodObjectSchema,
    S extends ZodObjectSchema
  >(node: AgentNode<C, I, O, S>) {
    throw new Error("not implemented");
  }

  compileGraph(graph: Graph) {
    const usedNodeIds = new Set(
      graph.edges.flatMap((edge) => {
        return [edge.from.node, edge.to.node];
      })
    );
    const actors = graph.nodes
      .filter((node) => usedNodeIds.has(node.id))
      .map((node) => {
        const nodeEdges = graph.edges.filter(
          (edge) => edge.to.node == node.id || edge.from.node == node.id
        );
        const Node = this.#agentNodesByName[node.type];
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
  #nodeTypeById: Record<string, string>;

  constructor(actors: ActorNode<any, ZodObjectSchema, ZodObjectSchema>[]) {
    this.#stream = new EventStream({
      run: {
        id: uniqueId(),
      },
    });

    this.#nodeTypeById = {};
    actors.forEach((actor) => {
      this.#nodeTypeById[actor.nodeId] = actor.node.metadata.id;
      actor.subscribe(this.#stream);
    });
  }

  trigger(node: { id: string }, input: any) {
    this.#stream.sendOutput(
      { id: node.id, type: this.#nodeTypeById[node.id] },
      input
    );
  }

  subscribe(callback: (event: any) => void) {
    this.#stream.subscribe((event) => {
      callback(event);
    });
  }
}

export { DefaultExecutorBuilder };
