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
import { ChatMessage } from "../repo/chatMessages";
import { ChatThread } from "./types";
import { llmDeltaToResponseBuilder } from "../../llm/utils";
import { createExtensionHandler } from "../../extensions/handler";
import { createQueryContextExtension } from "./llmQueryContext";
import { Search } from "@portal/workspace-sdk/llm/search";

async function generateLLMResponseStream(
  { ctx }: Pick<ProcedureRequest<Context, any>, "ctx" | "errors">,
  {
    thread,
    message,
    previousMessages,
    searchResults,
    context,
  }: {
    thread: ChatThread;
    message: ChatMessage;
    // old messages in the thread
    previousMessages: ChatMessage[];
    searchResults: Search.Response[];
    context?: {
      app: {
        id: string;
      };
      breadcrumbs: {}[];
    } | null;
  }
): Promise<Subject<any>> {
  const responseStream = new ReplaySubject<any>();

  const aiResponseTime = new Date();
  const aiResponseId = uniqueId();

  const openAiUserId = Buffer.from(
    JSON.stringify({ queryId: message.id })
  ).toString("base64");

  const extensionHandler = createExtensionHandler();
  const queryContextExtension = createQueryContextExtension(searchResults);
  const chatRequest = createRequestChain()
    .use(function setup({ request }) {
      request.stream = true;
    })
    // .use(function systemPromptGenerator({ request }) {
    //   request.messages = [
    //     {
    //       role: "system",
    //       content: generateSystemPrompt({
    //         // TODO(sagar): aggregate all chunks of the same document
    //         // into one section in the prompt
    //         documents: [
    //           // {
    //           //   content:
    //           //     "Current date and time is: " + new Date().toISOString(),
    //           // },
    //         ],
    //         has_functions: aiFunctions.length > 0,
    //       }),
    //     },
    //   ];
    // })
    .use(function setUserMessage({ request }) {
      request.addMessages(
        // @ts-expect-error
        ...previousMessages.map((m) => {
          return {
            content: m.message.content || "",
            role: m.role == "user" ? "user" : "assistant",
          };
        })
      );
      request.messages.push({
        role: "user",
        content: message.message.content!,
      });
    })
    .use(await queryContextExtension.middleware())
    .use(await extensionHandler.createRequestMiddleware())
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
            const followupRegex = new RegExp(
              /"_followup_question_":"(.*?)((?<!\\)(?=")|$)/
            );
            for await (const data of stream) {
              const { json } = data;
              if (json) {
                const delta = dlv(json, "choices.0.delta");
                builder.push(delta);

                const responseState = builder.build();
                const toolCallArgument = dlv(
                  responseState,
                  "tool_calls.0.function.arguments"
                );

                if (toolCallArgument) {
                  const matches = followupRegex.exec(toolCallArgument);
                  if (matches && matches[1]) {
                    responseStream.next({
                      ops: [
                        {
                          op: "replace",
                          path: [
                            "messages",
                            aiResponseId,
                            "message",
                            "content",
                          ],
                          value: matches[1],
                        },
                      ],
                    });
                  }
                }

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
        } else {
          throw new Error("not implemented");
        }

        streamSubject?.subscribe({
          error(err) {
            responseStream.complete();
          },
          async complete() {
            const aiResponse: any = await extensionHandler.parseResponse({
              data: {
                choices: [
                  {
                    message: builder.build(),
                  },
                ],
              },
              stream: undefined,
            });

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

            if (
              aiResponse.content == null &&
              aiResponse.tool_calls?.length > 0
            ) {
              if (aiResponse.tool_calls.length > 1) {
                console.error(
                  new Error("More than 1 tool calls not supported yet")
                );
              } else {
                const func = aiResponse.tool_calls[0];
                extensionHandler.startTask({
                  repos: {
                    tasks: ctx.repo.taskExecutions,
                    artifacts: ctx.repo.artifacts,
                  },
                  task: {
                    id: func.id,
                    threadId: thread.id,
                    messageId: aiResponseId,
                    name: func.function.name,
                    arguments: func.function.arguments,
                  },
                  context,
                });
                responseStream.next({
                  ops: [
                    {
                      op: "replace",
                      path: ["messages", aiResponseId, "message"],
                      value: aiResponse,
                    },
                  ],
                });
              }
            } else {
              // If a tool is called, only send the message with tool call
              // info after the task is executed. If a tool isn't called,
              // send the message immediately
              responseStream.next({
                ops: [
                  {
                    op: "replace",
                    path: ["messages", aiResponseId, "message"],
                    value: aiResponse,
                  },
                ],
              });
            }

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
            responseStream.complete();
          },
        });
      }
    )
    .catch((e) => {
      console.error(e);
    });
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
