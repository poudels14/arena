import z, { ZodObject, ZodRawShape } from "zod";
import { Context } from "./context";

type AtLeastOne<T> = {
  [K in keyof T]-?: Required<Pick<T, K>> & Partial<T>;
}[keyof T];

type AsyncGeneratorWithField<T> = AsyncGenerator<T, void, T>;

export type ZodObjectSchema =
  | ZodObject<ZodRawShape, "strip", z.ZodTypeAny, {}, {}>
  | ZodObject<ZodRawShape, "passthrough", z.ZodTypeAny, {}, {}>;

export type Metadata<
  Config extends ZodObjectSchema,
  Input extends ZodObjectSchema,
  Output extends ZodObjectSchema
> = {
  id: string;
  name: string;
  version: string;
  config: Config;
  input: Input;
  output: Output;
};

export const IS_AGENT_NODE = Symbol("_AGENT_NODE_");

export abstract class AgentNode<
  Config extends ZodObjectSchema,
  Input extends ZodObjectSchema,
  Output extends ZodObjectSchema
> {
  static [IS_AGENT_NODE]: boolean = true;

  abstract get metadata(): Metadata<Config, Input, Output>;

  abstract run(
    context: Context<z.infer<Config>, z.infer<Output>>,
    input: z.infer<Input>
  ): AsyncGeneratorWithField<AtLeastOne<z.infer<Output>>>;

  // TODO: in the future, to resolve only the output fields used by dependent nodes,
  // the interface can be updated such that AgentNode must implement `async resolve{OutputKey}`
  // That allows the dependencies to be pull based verses push-based and much more efficient
  // since the output field that isn't used doens't need to be resolved
}
