import dedent from "dedent";
import { ReplaySubject, Subject } from "rxjs";
import { ChatCompletionExecutor } from "@portal/cortex/executors";
import { ChatPromptTemplate, MessagesPlaceholder } from "@portal/cortex/prompt";
import {
  createStreamDeltaStringSubscriber,
  parseStringResponse,
} from "@portal/cortex/plugins/response";
import { Groq } from "@portal/cortex/integrations/models";

import { z } from "../core/zod";
import { AgentNode } from "../core/node";
import { Context } from "../core/context";

const config = z.object({
  systemPrompt: z.string().title("System Prompt"),
  temperature: z.number().title("temperature"),
  stream: z.boolean().optional().title("Stream"),
});

const input = z.object({
  query: z.string().title("Query"),
  context: z.string().optional().title("Context"),
  chatHistory: z.any().array().optional().title("Chat History"),
});

const output = z.object({
  stream: z.instanceof(Subject).title("Markdown stream"),
  markdown: z.string().title("Markdown"),
});

export class ChatCompletion extends AgentNode<
  typeof config,
  typeof input,
  typeof output
> {
  get metadata() {
    return {
      id: "@core/chat-completion",
      version: "0.0.1",
      name: "Chat completion",
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
    const { config } = context;
    const prompt = ChatPromptTemplate.fromMessages([
      [
        "system",
        dedent`
        {systemPrompt}

        {context}
        `,
      ],
      new MessagesPlaceholder("chatHistory"),
      ["human", "{query}"],
    ]);

    const chainModel = new Groq({
      model: "llama3-8b-8192",
    });

    const executor = new ChatCompletionExecutor({
      runnables: [prompt, chainModel],
      variables: {},
    });

    const stream = new ReplaySubject<any>();
    const res = executor.invoke({
      variables: {
        systemPrompt: config.systemPrompt,
        context: input.context || "",
        query: input.query,
        chatHistory: input.chatHistory || [],
      },
      config: {
        temperature: config.temperature || 0.7,
        stream: config.stream,
      },
      plugins: [
        createStreamDeltaStringSubscriber((chunk) => {
          stream.next({ type: "content/delta", delta: chunk });
        }),
      ],
    });

    yield { stream };
    const ctxt = await res;

    const response = parseStringResponse(ctxt);
    yield {
      markdown: response,
    };
  }
}
