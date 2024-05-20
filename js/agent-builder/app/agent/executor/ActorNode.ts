import { filter, mergeScan, of, tap, skipWhile } from "rxjs";
import {
  EventStream,
  AgentNode,
  ZodObjectSchema,
  EmptyAgentState,
  Edge,
} from "@portal/cortex/agent";

export class ActorNode<
  Config extends ZodObjectSchema,
  Input extends ZodObjectSchema,
  Output extends ZodObjectSchema,
  State extends ZodObjectSchema = EmptyAgentState
> {
  nodeId: string;
  node: AgentNode<Config, Input, Output, State>;
  #config: Config;
  #inputEdges: Edge[];
  #outputEdges: Edge[];

  constructor(
    nodeId: string,
    node: AgentNode<Config, Input, Output, State>,
    config: Config,
    // only pass in the input/output edges connected to this node
    edges: Edge[]
  ) {
    this.nodeId = nodeId;
    this.node = node;
    this.#config = config;
    this.#inputEdges = edges.filter((edge) => edge.to.node == nodeId);
    this.#outputEdges = edges.filter((edge) => edge.from.node == nodeId);
  }

  subscribe(stream: EventStream<any>) {
    const inputFieldsMapByOutputNode = this.#inputEdges.reduce((agg, cur) => {
      if (!agg[cur.from.node]) {
        agg[cur.from.node] = {};
      }
      agg[cur.from.node][cur.from.outputKey] = cur.to.inputKey;
      return agg;
    }, {} as any);

    const eventFilter = this.#inputEdges.map((edge) => edge.from.node);
    const requiredOutputFields = this.#outputEdges
      .filter((edge) => edge.from.node == this.nodeId)
      .map((edge) => edge.from.outputKey);

    const self = this;
    const context = {
      config: this.#config,
      requiredOutputFields,
      sendOutput(output: unknown) {
        stream.sendOutput(
          {
            id: self.nodeId,
            type: self.node.metadata.id,
          },
          output
        );
      },
    };

    // @ts-expect-error
    this.node.init(context);
    stream
      .pipe(
        filter(
          (event) =>
            eventFilter.includes(event.node.id) &&
            // filter out the output not used by this agent node
            inputFieldsMapByOutputNode[event.node.id]
        )
      )
      .pipe(
        mergeScan((acc, curr) => {
          const fieldMap = inputFieldsMapByOutputNode[curr.node.id];
          Object.entries(fieldMap).forEach(([outputKey, inputKey]: any) => {
            const value = curr.output[outputKey];
            if (value !== undefined) {
              acc[inputKey] = value;
              delete fieldMap[outputKey];
              if (Object.keys(fieldMap).length == 0) {
                delete inputFieldsMapByOutputNode[curr.node.id];
              }
            }
          });
          return of(acc);
        }, {} as any)
      )
      .pipe(
        tap((input) => {
          // @ts-expect-error
          this.node.onInputEvent(context, input);
        })
      )
      .pipe(
        skipWhile(() => {
          return Object.keys(inputFieldsMapByOutputNode).length > 0;
        })
      )
      .subscribe(async (input) => {
        // @ts-expect-error
        const generator = this.node.run(context, input);
        for await (const partialOutput of generator) {
          stream.sendOutput(
            { id: this.nodeId, type: this.node.metadata.id },
            partialOutput
          );
        }
      });
  }
}
