import React from "react";
import dedent from "dedent";
import { ReplaySubject, Subject } from "rxjs";
import { ChatCompletionExecutor } from "@portal/cortex/executors";
import { ChatPromptTemplate, MessagesPlaceholder } from "@portal/cortex/prompt";
import {
  createStreamDeltaStringSubscriber,
  parseStringResponse,
} from "@portal/cortex/plugins/response";
import { Groq } from "@portal/cortex/integrations/models";
import { z, AgentNode, Context } from "@portal/cortex/agent";
import { renderToString } from "react-dom/server";
import { HiSparkles } from "react-icons/hi";

const configSchema = z.object({
  systemPrompt: z
    .string()
    .default("You are an AI assistant.")
    .label("System Prompt")
    .uiSchema({
      type: "textarea",
    }),
  temperature: z.number({ coerce: true }).default(0.9).label("Temperature"),
  stream: z.boolean().default(false).label("Stream"),
});

const input = z.object({
  query: z.string().label("Query"),
  context: z.string().default("").label("Context"),
  chatHistory: z.any().array().default([]).label("Chat History"),
  tools: z.any().array().label("Tools"),
});

const output = z.object({
  stream: z.instanceof(Subject).label("Markdown stream"),
  markdown: z.string().label("Markdown"),
  tools: z.any().array().label("Tool calls"),
});

export class ChatCompletion extends AgentNode<
  typeof configSchema,
  typeof input,
  typeof output
> {
  get metadata() {
    return {
      id: "@core/chat-completion",
      version: "0.0.1",
      name: "Chat completion",
      icon: renderToString(React.createElement(HiSparkles, {})),
      config: configSchema,
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
    const config = configSchema.parse(context.config);
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
    stream.complete();
    yield {
      markdown: response,
    };
  }
}
