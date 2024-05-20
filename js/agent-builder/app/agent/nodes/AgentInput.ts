import { z, AgentNode, Context } from "@portal/cortex/agent";

const config = z.object({});

const input = z.object({});

const output = z
  .object({
    query: z.string().label("Query"),
  })
  .passthrough();

export class AgentInput extends AgentNode<
  typeof config,
  typeof input,
  typeof output
> {
  get metadata() {
    return {
      id: "@core/input",
      version: "0.0.1",
      name: "Agent Input",
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
    // TODO: add input validation node
    yield input;
  }
}
