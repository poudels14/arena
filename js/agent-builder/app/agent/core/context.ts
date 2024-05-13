import { EventStream } from "./stream";

export type Context<
  Config extends Record<string, unknown>,
  Output extends Record<string, unknown>
> = {
  config: Config;
  requiredOutputFields: (keyof Output)[];
  stream: EventStream<Output>;
};
