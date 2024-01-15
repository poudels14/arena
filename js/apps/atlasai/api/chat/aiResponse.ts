import { ProcedureRequest } from "@portal/server-core/router";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { OpenAIChat, chatCompletion } from "./OpenAI";
import { omit } from "lodash-es";
import { Context } from "../procedure";
import { DocumentEmbeddingsGenerator } from "../../llm/EmbeddingsGenerator";
import { generateSystemPrompt } from "./prompt";
import { mergeDelta } from "./llm";
import { ChatMessage } from "../repo/chatMessages";
import { ChatThread } from "./types";

const generateAIResponse = async (
  { ctx, errors }: Pick<ProcedureRequest<Context, any>, "ctx" | "errors">,
  {
    thread,
    isNewThread,
    message,
  }: { thread: ChatThread; isNewThread: boolean; message: ChatMessage }
) => {
  const generator = new DocumentEmbeddingsGenerator();
  // const embeddings = await generator.getTextEmbeddings([message.content]);
  // const { embeddings: documentEmbeddings } = await vectordb.searchCollection(
  //   "uploads",
  //   embeddings[0],
  //   4,
  //   {
  //     includeChunkContent: true,
  //     contentEncoding: "utf-8",
  //     minScore: 0.7,
  //   }
  // );

  // const { documents: pluginFunctions, embeddings: pluginEmbeddings } =
  //   await vectordb.searchCollection("plugin_functions", embeddings[0], 5, {
  //     includeChunkContent: true,
  //     contentEncoding: "utf-8",
  //     minScore: 0.75,
  //   });

  // const aiFunctions = pluginEmbeddings.map((r) => {
  //   const fn = JSON.parse(r.content);
  //   return {
  //     id: r.documentId,
  //     name: snakeCase(r.documentId),
  //     description: fn.description,
  //     parameters: fn.parameters,
  //   };
  // });
  const documentEmbeddings: any[] = [];
  const aiFunctions: any[] = [];
  const pluginFunctions: any[] = [];

  const aiResponseTime = new Date();
  const aiResponseId = uniqueId();

  const openAiUserId = Buffer.from(
    JSON.stringify({ queryId: message.id })
  ).toString("base64");

  const {
    request: llmQueryRequest,
    response: llmQueryResponse,
    stream: aiResponseStream,
  } = await chatCompletion({
    model: thread.metadata.ai.model,
    userId: openAiUserId,
    stream: true,
    messages: [
      {
        role: "system",
        content: generateSystemPrompt({
          // TODO(sagar): aggregate all chunks of the same document
          // into one section in the prompt
          documents: [
            {
              content: "Current date/time is: " + new Date().toISOString(),
            },
            // ...documentEmbeddings,
          ],
          has_functions: aiFunctions.length > 0,
        }),
      },
      {
        role: "user",
        content: message.message.content!,
      },
    ],
    functions: aiFunctions.length > 0 ? aiFunctions : undefined,
  });

  if (llmQueryResponse.status !== 200) {
    return errors.internalServerError("Error connection to the AI model");
  }

  let aiResponse: OpenAIChat.StreamResponseDelta = {};
  const stream = new ReadableStream({
    async start(controller) {
      if (isNewThread) {
        controller.enqueue(
          JSON.stringify({
            ops: [
              {
                op: "replace",
                path: ["threads", thread.id],
                value: thread,
              },
            ],
          })
        );
      }
      controller.enqueue(
        JSON.stringify({
          ops: [
            {
              op: "replace",
              path: ["messages", message.id],
              value: message,
            },
          ],
        })
      );

      controller.enqueue(
        JSON.stringify({
          ops: [
            {
              op: "replace",
              path: ["messages", aiResponseId],
              value: {
                id: aiResponseId,
                threadId: thread.id,
                parentId: message.id,
                role: "ai",
                message: aiResponse,
                userId: null,
                createdAt: aiResponseTime,
                metadata: {},
              },
            },
          ],
        })
      );

      try {
        let matchedFunctionCall: (typeof aiFunctions)[0] | undefined;
        for await (const data of aiResponseStream!) {
          if (data.json) {
            const { delta } = data.json.choices[0];
            aiResponse = mergeDelta(aiResponse, delta);
            if (delta.function_call?.name) {
              matchedFunctionCall = aiFunctions.find(
                (f) => f.name == delta.function_call?.name
              );
            }
            if (delta.content) {
              controller.enqueue(
                JSON.stringify({
                  ops: [
                    {
                      op: "add",
                      path: ["messages", aiResponseId, "message", "content"],
                      value: delta.content,
                    },
                  ],
                })
              );
            }
          }
        }

        const metadata: any = {
          documents: documentEmbeddings.map((r) =>
            omit(r, "content", "context")
          ),
        };

        controller.enqueue(
          JSON.stringify({
            ops: [
              {
                op: "replace",
                path: ["messages", aiResponseId, "message", "metadata"],
                value: metadata,
              },
            ],
          })
        );

        // const matchedFunction = pluginFunctions.find(
        //   (f) => f.id == matchedFunctionCall?.id
        // );
        // if (matchedFunction) {
        //   if (matchedFunction.metadata?.type == "workflow") {
        //     // Split at last index of '/'
        //     const matchedFunctionId = matchedFunction.id!;
        //     const slashIndex = matchedFunctionId.lastIndexOf("/");
        //     const pluginId = matchedFunctionId.substring(0, slashIndex);
        //     const workflowSlug = matchedFunctionId.substring(slashIndex + 1);
        //   }
        // }
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
              aiResponse,
            ],
          });
          if (title) {
            await ctx.repo.chatThreads.update({
              id: thread.id,
              title,
            });
            controller.enqueue(
              JSON.stringify({
                ops: [
                  {
                    op: "replace",
                    path: ["threads", thread.id, "title"],
                    value: title,
                  },
                ],
              })
            );
          }
        }
      } catch (e) {
        controller.error(e);
      } finally {
        controller.close();
      }
    },
  });

  return new Response(stream, {
    status: 200,
    headers: [["content-type", "text/event-stream"]],
  });
};

const generateThreadTitle = async (req: {
  model: string;
  userId: string;
  messages: any[];
}) => {
  const { response } = await chatCompletion({
    model: req.model,
    userId: req.userId,
    stream: false,
    messages: [
      {
        role: "system",
        content:
          "Generate a title of the following query in just a few words for the following text. don't use quotes. make it as short as possible",
      },
      ...req.messages.filter((m) => m.content),
    ],
  });

  return response.data.choices[0].message.content.replaceAll(/(^")|("$)/g, "");
};

export { generateAIResponse as generateAiResponse };
