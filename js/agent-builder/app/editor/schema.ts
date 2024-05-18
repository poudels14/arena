import z from "zod";

import { edge, node } from "../agent/core/graph";

const nodeWithUI = node.merge(
  z.object({
    ui: z.object({
      type: z.string(),
      label: z.string().optional(),
      position: z.object({
        x: z.number(),
        y: z.number(),
      }),
      draggable: z.boolean().optional(),
    }),
  })
);

const edgeWithUI = edge.merge(
  z.object({
    ui: z.object({
      type: z.enum(["step"]).optional(),
    }),
  })
);

const agentMetadata = z.object({
  graph: z.object({
    nodes: z.array(nodeWithUI),
    edges: z.array(edgeWithUI),
  }),
});

export { nodeWithUI, edgeWithUI, agentMetadata };

export type NodeWithUI = z.infer<typeof nodeWithUI>;
export type EdgeWithUI = z.infer<typeof edgeWithUI>;
export type AgentMetadata = z.infer<typeof agentMetadata>;
