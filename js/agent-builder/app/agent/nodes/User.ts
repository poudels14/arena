import { z } from "../core/zod";
import { AgentNode } from "../core/node";
import { Context } from "../core/context";
import { Subject } from "rxjs";

const config = z.object({});

const input = z.object({
  stream: z.instanceof(Subject).label("Stream"),
});

const output = z.object({
  stream: z.instanceof(Subject).label("Stream"),
});

export class User extends AgentNode<
  typeof config,
  typeof input,
  typeof output
> {
  get metadata() {
    return {
      id: "@core/user",
      version: "0.0.1",
      name: "User",
      config,
      input,
      output,
    };
  }

  async *run(
    context: Context<
      z.infer<typeof this.metadata.config>,
      z.infer<typeof this.metadata.output>
    >,
    input: z.infer<typeof this.metadata.input>
  ) {
    yield input;
  }
}
