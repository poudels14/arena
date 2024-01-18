import { zod as z, zodToJsonSchema } from "@portal/sdk";
import { Middleware } from "@portal/sdk/llm/chain";
import { Subject } from "rxjs";
import dlv from "dlv";
import cleanSet from "clean-set";
import { klona } from "klona";
import { merge } from "lodash-es";
import { llmDeltaToResponseBuilder } from "../llm/utils";

import { manifest as timer } from "./clock/timer";
import { createRepo } from "~/api/repo/tasks";
import { dset } from "dset";

const followUpQuestionSchema = z.object({
  _followup_question_: z
    .string()
    .optional()
    .describe(
      "A follow up question to ask the user in order to gather the information needed to perform an actions. Only use this if you need more information from the user, else skip this field."
    ),
});

const TASKS = [timer];

function createExtensionHandler() {
  return {
    async createRequestMiddleware(): Promise<Middleware> {
      return function addFunctions({ request }) {
        request.addFunction({
          name: timer.name,
          description: timer.description,
          parameters: zodToJsonSchema(
            timer.schema.merge(followUpQuestionSchema)
          ),
        });
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
              resolve(createFinalPayload(responseBuilder.build()));
            },
          });
        } else if (result.data) {
          const content = dlv(result.data, "choices.0.message.content");
          const tool_calls = dlv(result.data, "choices.0.message.tool_calls");
          resolve(
            createFinalPayload({
              role: dlv(result.data, "choices.0.message.role") || "assistant",
              content,
              tool_calls,
            })
          );
        } else {
          throw new Error("Either stream or data expected");
        }
      });
    },
    async startTask(options: {
      repo: ReturnType<typeof createRepo>;
      task: {
        id: string;
        threadId: string;
        messageId: string;
        name: string;
        arguments: any;
      };
    }) {
      const { repo, task } = options;
      const selectedTask = TASKS.find((t) => t.name == task.name);

      if (!selectedTask) return;
      await repo.insert({
        id: task.id,
        taskId: task.name,
        threadId: task.threadId,
        messageId: task.messageId,
        metadata: { arguments: task.arguments },
        state: {},
      });

      await selectedTask.start({
        input: task.arguments,
        async prompt(message) {
          throw new Error("not implemented");
        },
        async setState(state) {
          await repo.update({
            id: task.id,
            state,
          });
        },
      });
    },
  };
}

export { createExtensionHandler };
