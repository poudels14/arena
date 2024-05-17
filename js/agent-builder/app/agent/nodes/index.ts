import { glob } from "glob";

import { AgentNode, IS_AGENT_NODE, ZodObjectSchema } from "../core/node";

const listNodes = async () => {
  const jsfiles = await glob(import.meta.dirname + "/**.ts", {
    ignore: "**/index.ts",
  });

  const imports = await Promise.all(jsfiles.map((file) => import(file)));

  const agentNodes: any[] = imports.flatMap((m) => {
    return Object.values(m).filter((e: any) => {
      return e[IS_AGENT_NODE];
    });
  });

  return agentNodes.map((Node: any) => {
    const node: AgentNode<ZodObjectSchema, ZodObjectSchema, ZodObjectSchema> =
      new Node();

    const config = node.metadata.config._def.shape();
    const inputs = node.metadata.input._def.shape();
    const outputs = node.metadata.output._def.shape();
    return {
      id: node.metadata.id,
      version: node.metadata.version,
      name: node.metadata.name,
      config: Object.entries(config).map(([key, shape]) => {
        return {
          id: key,
          title: shape._def.title as string,
        };
      }),
      inputs: Object.entries(inputs).map(([key, shape]) => {
        return {
          id: key,
          title: shape._def.title as string,
        };
      }),
      outputs: Object.entries(outputs).map(([key, shape]) => {
        return {
          id: key,
          title: shape._def.title as string,
        };
      }),
    };
  });
};

export { listNodes };
