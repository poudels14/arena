import { z } from "zod";
import { uniqueId } from "@portal/cortex/utils/uniqueId";
import { AgentNode } from "../core/node";
import { Context } from "../core/context";

const config = z.object({});

const input = z.object({
  threadId: z.string().nullable(),
});

const output = z.object({
  threadId: z.string(),
  history: z.any().array(),
});

export class ChatThread extends AgentNode<
  typeof config,
  typeof input,
  typeof output
> {
  get metadata() {
    return {
      name: "@core/chat-thread",
      version: "0.0.1",
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
    yield {
      threadId: input.threadId || uniqueId(),
    };

    if (context.requiredOutputFields.includes("history")) {
      // TODO: load history and return
      yield {
        history: [
          {
            role: "user",
            content: "I am Tom",
          },
        ],
      };
    }
  }
}
