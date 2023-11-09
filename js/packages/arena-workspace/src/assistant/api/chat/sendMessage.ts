import { ProcedureCallbackArgs } from "@arena/runtime/server";
import { OpenAIChat, chatCompletion } from "./OpenAI";
import { omit, snakeCase, pick } from "lodash-es";
import { AssistantContext } from "../procedure";
import { DocumentEmbeddingsGenerator } from "../../llm/EmbeddingsGenerator";
import { uniqueId } from "~/utils/uniqueId";
import { generateSystemPrompt } from "./prompt";
import { mergeDelta } from "./llm";
import { ChatChannel, ChatThread } from "./types";

const generateAiResponse = async (
  {
    ctx,
    errors,
  }: Pick<ProcedureCallbackArgs<AssistantContext>, "ctx" | "errors">,
  channel: any,
  thread: any,
  message: { id: string; content: string }
) => {
  const { default: sqlite, vectordb } = ctx.dbs;
  const now = new Date();

  const threadMessages = await sqlite.query(
    `SELECT * FROM chat_messages WHERE thread_id = ?`,
    [thread.id]
  );

  console.log("threadMessages =", threadMessages);

  await sqlite.query(
    `INSERT INTO chat_messages(id, channel_id, thread_id, role, user_id, message, timestamp)
    VALUES (?,?,?,?,?,?,?)`,
    [
      message.id,
      channel.id,
      thread.id,
      "user",
      ctx.user?.id,
      JSON.stringify({
        content: message.content,
      }),
      now.getTime(),
    ]
  );

  const generator = new DocumentEmbeddingsGenerator();
  const embeddings = await generator.getTextEmbeddings([message.content]);
  const { embeddings: documentEmbeddings } = await vectordb.searchCollection(
    "uploads",
    embeddings[0],
    4,
    {
      includeChunkContent: true,
      contentEncoding: "utf-8",
      minScore: 0.7,
    }
  );

  const { documents: pluginFunctions, embeddings: pluginEmbeddings } =
    await vectordb.searchCollection("plugin_functions", embeddings[0], 5, {
      includeChunkContent: true,
      contentEncoding: "utf-8",
      minScore: 0.75,
    });

  const aiFunctions = pluginEmbeddings.map((r) => {
    const fn = JSON.parse(r.content);
    return {
      id: r.documentId,
      name: snakeCase(r.documentId),
      description: fn.description,
      parameters: fn.parameters,
    };
  });

  const aiResponseTime = new Date();
  const aiResponseId = uniqueId();

  const openAiUserId = encodeToBase64(
    Buffer.from(JSON.stringify({ queryId: message.id }))
  );

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
            ...documentEmbeddings,
          ],
          has_functions: aiFunctions.length > 0,
        }),
      },
      {
        role: "user",
        content: message.content,
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
      controller.enqueue(
        JSON.stringify({
          message: {
            id: aiResponseId,
            timestamp: aiResponseTime.getTime(),
          },
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
            controller.enqueue(
              JSON.stringify({
                message: {
                  id: aiResponseId,
                  delta,
                },
              })
            );
          }
        }

        const metadata: any = {
          documents: documentEmbeddings.map((r) =>
            omit(r, "content", "context")
          ),
        };

        controller.enqueue(
          JSON.stringify({
            message: {
              id: aiResponseId,
              metadata,
            },
          })
        );

        const matchedFunction = pluginFunctions.find(
          (f) => f.id == matchedFunctionCall?.id
        );

        if (matchedFunction) {
          if (matchedFunction.metadata?.type == "workflow") {
            controller.enqueue(
              JSON.stringify({
                thread: {
                  id: thread.id,
                  blockedBy: "workflow",
                },
              })
            );

            // Split at last index of '/'
            const matchedFunctionId = matchedFunction.id!;
            const slashIndex = matchedFunctionId.lastIndexOf("/");
            const pluginId = matchedFunctionId.substring(0, slashIndex);
            const workflowSlug = matchedFunctionId.substring(slashIndex + 1);

            const {
              rows: [plugin],
            } = await sqlite.query<{ id: string; version: string }>(
              `SELECT * FROM installed_plugins WHERE id = ?`,
              [pluginId]
            );

            const workflowRun = await createAndRunWorkflow(
              ctx,
              channel,
              thread,
              {
                plugin,
                slug: workflowSlug,
              }
            );

            console.log("workflowRun =", workflowRun);

            metadata.workflow = { id: workflowRun.id };
          }
        }

        console.log("Saving chat metadata =", metadata);
        await sqlite.query(
          `INSERT INTO chat_messages
            (id, channel_id, thread_id, parent_id, role, message, metadata, timestamp)
            VALUES (?,?,?,?,?,?,?,?)`,
          [
            aiResponseId,
            channel.id,
            thread.id,
            message.id,
            "ai",
            JSON.stringify(aiResponse),
            JSON.stringify(metadata),
            aiResponseTime.getTime(),
          ]
        );

        if (!thread.title) {
          const title = await generateThreadTitle({
            model: thread.metadata.ai.model,
            userId: openAiUserId,
            messages: [
              {
                role: "user",
                content: message.content,
              },
              aiResponse,
            ],
          });
          if (title) {
            await sqlite.query(
              `UPDATE chat_threads SET title = ? WHERE id = ? AND channel_id = ?`,
              [title, thread.id, channel.id]
            );
            controller.enqueue(
              JSON.stringify({
                thread: {
                  id: thread.id,
                  title,
                },
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

const createAndRunWorkflow = async (
  ctx: AssistantContext,
  channel: ChatChannel,
  thread: ChatThread,
  workflow: {
    plugin: { id: string; version: string };
    slug: string;
  }
) => {
  const workflowRunId = uniqueId();
  const { default: sqlite } = ctx.dbs;
  await sqlite.transaction(async () => {
    const {
      rows: [{ metadata: rawMetadata }],
    } = await sqlite.query<{ metadata: string }>(
      `SELECT metadata FROM chat_threads WHERE id = ? AND channel_id = ?`,
      [thread.id, channel.id]
    );
    const metadata = JSON.parse(rawMetadata) as ChatThread["metadata"];

    metadata.activeWorkflow = {
      id: workflowRunId,
    };

    await sqlite.query(
      `INSERT INTO workflow_runs
      (id, channel_id, thread_id, plugin, workflow_slug, status, triggered_at)
      VALUES (?,?,?,?,?,?,?)`,
      [
        workflowRunId,
        channel.id,
        thread.id,
        JSON.stringify(pick(workflow.plugin, "id", "version")),
        workflow.slug,
        "CREATED",
        new Date().getTime(),
      ]
    );

    await sqlite.query(
      `UPDATE chat_threads SET blocked_by = 'workflow', metadata = ?
      WHERE id = ? AND channel_id = ?`,
      [JSON.stringify(metadata), thread.id, channel.id]
    );
  });

  return {
    id: workflowRunId,
  };
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

export { generateAiResponse };
