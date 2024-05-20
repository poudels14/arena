import { glob } from "glob";
import { omit } from "lodash-es";
import zodToJsonSchema from "zod-to-json-schema";
import {
  AgentNode,
  IS_AGENT_NODE,
  ZodObjectSchema,
} from "@portal/cortex/agent";

const listNodes = async () => {
  const jsfiles = await glob(import.meta.dirname + "/**/*.{ts,tsx}", {
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
      icon: node.metadata.icon,
      config: Object.entries(config).map(([key, shape]) => {
        // @ts-expect-error
        const schema = omit(zodToJsonSchema(shape), "$schema");
        if (schema.default == undefined) {
          throw new Error(`default value missing from config field: "${key}"`);
        }
        return {
          id: key,
          label: (shape._def.label as string) || key,
          schema,
          ui: {
            type: shape._def.uiSchema?.type,
          },
        };
      }),
      inputs: Object.entries(inputs).map(([key, shape]) => {
        const schema = omit(zodToJsonSchema(shape), "$schema");
        return {
          id: key,
          label: (shape._def.label as string) || key,
          schema,
        };
      }),
      outputs: Object.entries(outputs).map(([key, shape]) => {
        const schema = omit(zodToJsonSchema(shape), "$schema");
        return {
          id: key,
          label: (shape._def.label as string) || key,
          schema,
        };
      }),
    };
  });
};

export { listNodes };
