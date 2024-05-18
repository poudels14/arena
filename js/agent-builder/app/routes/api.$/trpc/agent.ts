import fs from "fs/promises";
import path from "path";
import { z } from "zod";
import { pick } from "lodash-es";

import { publicProcedure } from "./trpc";
import { AgentMetadata } from "../../../editor/schema";

const node = z.object({
  id: z.string(),
  data: z.object({
    label: z.string(),
    type: z.string(),
    config: z.any(),
  }),
  position: z.object({
    x: z.number(),
    y: z.number(),
  }),
  type: z.string(),
  draggable: z.boolean().optional(),
});

const edge = z.object({
  id: z.string(),
  type: z.string().optional(),
  source: z.string(),
  sourceHandle: z.string(),
  target: z.string(),
  targetHandle: z.string(),
});

const graph = z.object({
  nodes: z.array(node),
  edges: z.array(edge),
});

const getAgent = publicProcedure
  .input(z.object({ id: z.string() }))
  .output(
    z.object({
      graph,
    })
  )
  .query(async ({ input }) => {
    const agent: AgentMetadata = await loadAgentGraph(input.id);
    const nodes = agent.graph.nodes.map((n) => {
      return {
        id: n.id,
        data: {
          label: n.ui.label || "Untitled",
          type: n.type,
          config: n.config,
        },
        position: n.ui.position || { x: 0, y: 0 },
        type: n.ui.type || "agentNode",
        ...pick(n.ui, "draggable"),
      };
    });

    const edges = agent.graph.edges.map((e) => {
      return {
        id: e.id,
        type: e.ui?.type,
        source: e.from.node,
        sourceHandle: e.from.outputKey,
        target: e.to.node,
        targetHandle: e.to.inputKey,
      };
    });

    return {
      graph: {
        nodes,
        edges,
      },
    };
  });

const updateAgent = publicProcedure
  .input(
    z.object({
      id: z.string(),
      graph,
    })
  )
  .mutation(async ({ input }) => {
    const updatedAgent: AgentMetadata = {
      graph: {
        nodes: input.graph.nodes.map((n) => {
          return {
            id: n.id,
            type: n.data.type,
            config: n.data.config,
            ui: {
              label: n.data.label,
              type: n.type,
              position: n.position,
              draggable: n.draggable,
            },
          };
        }),
        edges: input.graph.edges.map((e) => {
          return {
            id: e.id,
            from: {
              node: e.source,
              outputKey: e.sourceHandle,
            },
            to: {
              node: e.target,
              inputKey: e.targetHandle,
            },
            ui: {
              type: e.type as any,
            },
          };
        }),
      },
    };

    await fs.writeFile(
      path.join(import.meta.dirname, `../${input.id}.json`),
      JSON.stringify(updatedAgent, null, 2),
      {
        encoding: "utf-8",
      }
    );
    return { success: true };
  });

const loadAgentGraph = async (id: string) => {
  const agent: AgentMetadata = await fs
    .readFile(path.join(import.meta.dirname, `../${id}.json`), "utf-8")
    .then((data) => JSON.parse(data));

  return agent;
};

export { getAgent, updateAgent, loadAgentGraph };
