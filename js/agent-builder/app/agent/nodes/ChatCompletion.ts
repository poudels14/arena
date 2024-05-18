import dedent from "dedent";
import { ReplaySubject, Subject } from "rxjs";
import { ChatCompletionExecutor } from "@portal/cortex/executors";
import { ChatPromptTemplate, MessagesPlaceholder } from "@portal/cortex/prompt";
import {
  createStreamDeltaStringSubscriber,
  parseStringResponse,
} from "@portal/cortex/plugins/response";
import { Groq } from "@portal/cortex/integrations/models";
import { renderToString } from "react-dom/server";
import { HiSparkles } from "react-icons/hi";

import { z } from "../core/zod";
import { AgentNode } from "../core/node";
import { Context } from "../core/context";
import React from "react";

const config = z.object({
  systemPrompt: z
    .string()
    .default("You are an AI assistant.")
    .label("System Prompt")
    .uiSchema({
      type: "textarea",
    }),
  temperature: z.number().default(0.9).label("Temperature"),
  stream: z.boolean().default(false).label("Stream"),
});

const input = z.object({
  query: z.string().label("Query"),
  context: z.string().default("").label("Context"),
  chatHistory: z.any().array().default([]).label("Chat History"),
});

const output = z.object({
  stream: z.instanceof(Subject).label("Markdown stream"),
  markdown: z.string().label("Markdown"),
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
      icon: renderToString(React.createElement(HiSparkles, {})),
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
