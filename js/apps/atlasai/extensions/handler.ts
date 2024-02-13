import { zod as z, zodToJsonSchema } from "@portal/sdk";
import { Middleware } from "@portal/sdk/llm/chain";
import { Subject } from "rxjs";
import dlv from "dlv";
import cleanSet from "clean-set";
import { klona } from "klona";
import { merge } from "lodash-es";
import { serializeError } from "serialize-error";

import { llmDeltaToResponseBuilder } from "../llm/utils";

import { manifest as timer } from "./clock/timer";
import { manifest as interpreter } from "./interpreter";
import { createRepo as createTaskRepo } from "~/api/repo/tasks";
import { createRepo as createArtifactRepo } from "~/api/repo/artifacts";
import { dset } from "dset";
import { env } from "~/api/env";

import type { Artifact } from "./types";
import path from "path";

const followUpQuestionSchema = z.object({
  _followup_question_: z
    .string()
    .optional()
    .describe(
      "A follow up question to ask the user in order to gather the information needed to perform an actions. Only use this if you need more information from the user, else skip this field."
    ),
});

const TOOLS = [timer, interpreter];

function createExtensionHandler() {
  return {
    async createRequestMiddleware(): Promise<Middleware> {
      return function addFunctions({ request }) {
        for (const tool of TOOLS) {
          const parameters = tool.config?.setup?.disableFollowUp
            ? tool.schema
            : tool.schema.merge(followUpQuestionSchema);
          request.addFunction({
            name: tool.name,
            description: tool.description,
            parameters: zodToJsonSchema(parameters),
          });
        }
      };
    },
    async parseResponse(result: { data?: any; stream?: Subject<any> }) {
      return new Promise(async (resolve) => {
        const createFinalPayload = (message: {
          role: string;
          content: string | null;
          tool_calls: any[] | null;
        }) => {
          const toolArgs = dlv(message, "tool_calls.0.function.arguments");
          if (toolArgs) {
            // TODO: catch error if args isn't a valid JSON
            const args = JSON.parse(toolArgs);

            if (args._followup_question_) {
              if (message.content) {
                throw new Error(
                  "Only either message content or follow question expected to be set but both are set"
                );
              }
              const cleanMessage = klona(message);
              // remove the id from the tool call if there's a follow up since
              // the tool don't actually get called
              dset(cleanMessage, "tool_calls.0.id", undefined);
              dset(
                cleanMessage,
                "tool_calls.0.function.arguments",
                cleanSet(args, "_followup_question_", undefined)
              );
              return merge(cleanMessage, {
                content: args._followup_question_,
                followup: true,
              });
            }
            return cleanSet(message, "tool_calls.0.function.arguments", args);
          }
          return message;
        };

        if (result.stream) {
          const responseBuilder = llmDeltaToResponseBuilder();
          result.stream.subscribe({
            next(payload) {
              responseBuilder.push(dlv(payload, "choices.0.delta"));
            },
            complete() {
              try {
                resolve(createFinalPayload(responseBuilder.build()));
              } catch (e) {
                resolve({
                  raw: responseBuilder.build(),
                });
              }
            },
          });
        } else if (result.data) {
          const content = dlv(result.data, "choices.0.message.content");
          const tool_calls = dlv(result.data, "choices.0.message.tool_calls");

          try {
            resolve(
              createFinalPayload({
                role: dlv(result.data, "choices.0.message.role") || "assistant",
                content,
                tool_calls,
              })
            );
          } catch (e) {
            resolve({
              raw: {
                role: dlv(result.data, "choices.0.message.role") || "assistant",
                content,
                tool_calls,
              },
            });
          }
        } else {
          throw new Error("Either stream or data expected");
        }
      });
    },
    async startTask(options: {
      repos: {
        tasks: ReturnType<typeof createTaskRepo>;
        artifacts: ReturnType<typeof createArtifactRepo>;
      };
      task: {
        id: string;
        threadId: string;
        messageId: string;
        name: string;
        arguments: any;
      };
    }) {
      const { repos, task } = options;
      const selectedTask = TOOLS.find((t) => t.name == task.name);

      if (!selectedTask) return;
      const taskMetadata = { arguments: task.arguments };
      await repos.tasks.insert({
        id: task.id,
        taskId: task.name,
        threadId: task.threadId,
        messageId: task.messageId,
        metadata: taskMetadata,
        state: {},
      });

      await selectedTask
        .start({
          args: task.arguments,
          env: {
            PORTAL_CODE_INTERPRETER_HOST: env.PORTAL_CODE_INTERPRETER_HOST,
          },
          utils: {
            async uploadArtifact(artifact: Artifact) {
              const name = path.basename(artifact.path);
              await repos.artifacts.insert({
                id: artifact.id,
                name,
                threadId: task.threadId,
                messageId: task.messageId,
                size: artifact.size,
                file: {
                  content: artifact.content,
                },
                createdAt: new Date(),
                metadata: {},
              });
              return { id: artifact.id, name };
            },
          },
          async prompt(message) {
            throw new Error("not implemented");
          },
          async setStatus(status) {
            await repos.tasks.update({
              id: task.id,
              status,
            });
          },
          async setState(state, status) {
            await repos.tasks.update({
              id: task.id,
              status,
              state,
            });
          },
        })
        .catch(async (e) => {
          await repos.tasks.update({
            id: task.id,
            status: "ERROR",
            metadata: {
              ...taskMetadata,
              error: serializeError(e),
            },
          });
        });
    },
  };
}

export { createExtensionHandler };
