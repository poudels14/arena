import { uniqueId } from "@portal/cortex/utils/uniqueId";

import { z } from "../core/zod";
import { AgentNode } from "../core/node";
import { Context } from "../core/context";

const config = z.object({});

const input = z.object({
  threadId: z.string().nullable().label("Thread id"),
});

const output = z.object({
  threadId: z.string().label("Thread id"),
  history: z.any().array().label("History"),
});

export class ChatThread extends AgentNode<
  typeof config,
  typeof input,
  typeof output
> {
  get metadata() {
    return {
      id: "@core/chat-thread",
      version: "0.0.1",
      name: "Chat thread",
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
