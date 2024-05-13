import { filter, mergeScan, of, skipWhile } from "rxjs";
import { Edge } from "../core/graph";
import { AgentNode, ZodObjectSchema } from "../core/node";
import { EventStream } from "../core/stream";

export class ActorNode<
  Config extends ZodObjectSchema,
  Input extends ZodObjectSchema,
  Output extends ZodObjectSchema
> {
  nodeId: string;
  node: AgentNode<Config, Input, Output>;
  config: Config;
  inputEdges: Edge[];
  outputEdges: Edge[];

  constructor(
    nodeId: string,
    node: AgentNode<Config, Input, Output>,
    config: Config,
    // only pass in the input/output edges connected to this node
    edges: Edge[]
  ) {
    this.nodeId = nodeId;
    this.node = node;
    this.config = config;
    this.inputEdges = edges.filter((edge) => edge.to.node == nodeId);
    this.outputEdges = edges.filter((edge) => edge.from.node == nodeId);
  }

  subscribe(stream: EventStream<any>) {
    const inputFieldsMapByOutputNode = this.inputEdges.reduce((agg, cur) => {
      if (!agg[cur.from.node]) {
        agg[cur.from.node] = {};
      }
      agg[cur.from.node][cur.from.outputKey] = cur.to.inputKey;
      return agg;
    }, {} as any);

    const eventFilter = this.inputEdges.map((edge) => edge.from.node);
    stream
      .pipe(filter((event) => eventFilter.includes(event.node.id)))
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
        skipWhile(() => {
          return Object.keys(inputFieldsMapByOutputNode).length > 0;
        })
      )
      .subscribe(async (input) => {
        const requiredOutputFields = this.outputEdges
          .filter((edge) => edge.from.node == this.nodeId)
          .map((edge) => edge.from.outputKey);
        const generator = this.node.run(
          {
            stream,
            config: this.config,
            // @ts-expect-error
            requiredOutputFields,
          },
          input
        );
        // @ts-expect-error
        for await (const partialOutput of generator) {
          stream.sendOutput({ id: this.nodeId }, partialOutput);
        }
      });
  }
}
