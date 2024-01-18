import { Subject, ReplaySubject } from "rxjs";
import { ProcedureRequest } from "@portal/server-core/router";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import {
  createOpenAIProvider,
  createRequestChain,
} from "@portal/sdk/llm/chain";
import { jsonStreamToAsyncIterator } from "@portal/sdk/utils/stream";
import dlv from "dlv";
import { pick } from "lodash-es";
import { Context } from "../procedure";
import { DocumentEmbeddingsGenerator } from "../../llm/EmbeddingsGenerator";
import { generateSystemPrompt } from "./prompt";
import { ChatMessage } from "../repo/chatMessages";
import { ChatThread } from "./types";
import { llmDeltaToResponseBuilder } from "../../llm/utils";

async function generateLLMResponseStream(
  { ctx, errors }: Pick<ProcedureRequest<Context, any>, "ctx" | "errors">,
  { thread, message }: { thread: ChatThread; message: ChatMessage }
): Promise<Subject<any>> {
  const responseStream = new ReplaySubject<any>();

  // const generator = new DocumentEmbeddingsGenerator();
  const aiFunctions: any[] = [];

  const aiResponseTime = new Date();
  const aiResponseId = uniqueId();

  const openAiUserId = Buffer.from(
    JSON.stringify({ queryId: message.id })
  ).toString("base64");

  const chatRequest = createRequestChain()
    .use(function setup({ request }) {
      request.stream = true;
    })
    .use(function systemPromptGenerator({ request }) {
      request.messages = [
        {
          role: "system",
          content: generateSystemPrompt({
            // TODO(sagar): aggregate all chunks of the same document
            // into one section in the prompt
            documents: [
              {
                content: "Current date/time is: " + new Date().toISOString(),
              },
            ],
            has_functions: aiFunctions.length > 0,
          }),
        },
      ];
    })
    .use(function setUserMessage({ request }) {
      request.messages.push({
        role: "user",
        content: message.message.content!,
      });
    })
    .use(
      createOpenAIProvider({
        model: thread.metadata.ai.model,
      })
    );

  chatRequest
    .invoke({
      user: openAiUserId,
    })
    .then(
      async ({
        result: { request: llmQueryRequest, response: llmQueryResponse },
      }) => {
        // TODO: add error message to the db
        // if (llmQueryResponse.status !== 200) {
        //   return errors.internalServerError("Error connection to the AI model");
        // }

        responseStream.next({
          ops: [
            {
              op: "replace",
              path: ["messages", aiResponseId],
              value: {
                id: aiResponseId,
                threadId: thread.id,
                parentId: message.id,
                role: "ai",
                message: {},
                userId: null,
                createdAt: aiResponseTime,
                metadata: {},
              },
            },
          ],
        });

        const stream = llmQueryRequest.stream
          ? jsonStreamToAsyncIterator(llmQueryResponse.body)
          : undefined;

        const streamSubject = stream ? new ReplaySubject<any>() : undefined;
        const builder = llmDeltaToResponseBuilder();
        if (stream) {
          (async function streamRunner() {
            for await (const { json } of stream) {
              if (json) {
                const delta = dlv(json, "choices.0.delta");
                builder.push(delta);
                streamSubject?.next(json);
                if (delta.content) {
                  responseStream.next({
                    ops: [
                      {
                        op: "add",
                        path: ["messages", aiResponseId, "message", "content"],
                        value: delta.content,
                      },
                    ],
                  });
                }
              }
            }
            streamSubject?.complete();
          })();
        }

        streamSubject?.subscribe({
          async complete() {
            let aiResponse = builder.build();
            await ctx.repo.chatMessages.insert({
              id: aiResponseId,
              threadId: thread.id,
              parentId: message.id,
              role: "ai",
              message: aiResponse,
              userId: null,
              createdAt: aiResponseTime,
              metadata: {},
            });

            if (!thread.title) {
              const title = await generateThreadTitle({
                model: thread.metadata.ai.model,
                userId: openAiUserId,
                messages: [
                  {
                    role: "user",
                    content: message.message.content!,
                  },
                  pick(aiResponse, "role", "content"),
                ],
              });
              if (title) {
                await ctx.repo.chatThreads.update({
                  id: thread.id,
                  title,
                });
                responseStream.next({
                  ops: [
                    {
                      op: "replace",
                      path: ["threads", thread.id, "title"],
                      value: title,
                    },
                  ],
                });
              }
            }
          },
        });
      }
    );
  return responseStream;
}

const generateThreadTitle = async (req: {
  model: string;
  userId: string;
  messages: any[];
}) => {
  const request = createRequestChain()
    .use(function enableStreaming({ request }) {
      request.stream = false;
      request.messages = [
        {
          role: "system",
          content:
            "Generate a title of the following query in just a few words for the following text. don't use quotes. make it as short as possible",
        },
        ...req.messages.filter((m) => m.content),
      ];
    })
    .use(
      createOpenAIProvider({
        model: req.model,
      })
    );

  const { result } = await request.invoke({
    user: req.userId,
  });
  return result.response.data.choices[0].message.content.replaceAll(
    /(^")|("$)/g,
    ""
  );
};

export { generateLLMResponseStream };
