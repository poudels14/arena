import { omit, pick, snakeCase, uniqBy } from "lodash-es";
import { p } from "../../procedure";
import { OpenAIChat, chatCompletion } from "./OpenAI";
import { generateSystemPrompt } from "./prompt";
import { DocumentEmbeddingsGenerator } from "../../EmbeddingsGenerator";
import { uniqueId } from "../../utils";
import { mergeDelta } from "./llm";

const listChannels = p.query(async ({ ctx }) => {
  const { rows } = await ctx.dbs.default.query(`SELECT * FROM chat_channels`);
  return rows;
});

const listMessages = p.query(async ({ ctx, params }) => {
  const { rows: messages } = await ctx.dbs.default.query(
    `SELECT * FROM chat_messages where channel_id = ? ORDER BY timestamp`,
    [params.channelId]
  );
  return messages.map((m: any) => {
    const metadata = JSON.parse(m.metadata);
    return {
      ...pick(
        m,
        "id",
        "threadId",
        "parentId",
        "role",
        "userId",
        "model",
        "timestamp"
      ),
      message: JSON.parse(m.message),
      metadata: {
        documents: uniqBy(
          metadata?.documents.map((d: any) => pick(d, "documentId", "score")),
          (d: any) => d.documentId
        ),
      },
    };
  });
});

const sendMessage = p.mutate(async ({ ctx, params, req, errors }) => {
  const { vectordb } = ctx.dbs;
  let request: {
    id: string;
    message: string;
  };
  try {
    request = await req.json();
  } catch (e) {
    return "Error parsing request body";
  }

  const channelId = params.channelId;
  if (!request.message) {
    errors.badRequest("Message can't be empty");
  }

  // TODO(sagar): abstract several steps and use middleware like system
  // so that it's easier to build plugins for chat system itself. For example,
  // there can be a plugin to augment prompts to provide better results, or
  // a plugin to provide additional context based on the prompt, or a plugin
  // to built agent like multi-step llm querries
  request.id = request.id || uniqueId();

  const { rows: channels } = await ctx.dbs.default.query(
    `SELECT * FROM chat_channels WHERE id = ?`,
    [channelId]
  );
  if (channels.length == 0) {
    await ctx.dbs.default.query(`INSERT INTO chat_channels(id) VALUES (?)`, [
      channelId,
    ]);
  }

  await ctx.dbs.default.query(
    `INSERT INTO chat_messages(id, channel_id, role, message, timestamp) VALUES (?,?,?,?,?)`,
    [
      request.id,
      channelId,
      ctx.user?.id || "user",
      JSON.stringify({
        content: request.message,
      }),
      new Date().getTime(),
    ]
  );

  const generator = new DocumentEmbeddingsGenerator();
  const embeddings = await generator.getTextEmbeddings([request.message]);
  const documentsSearchResults = await vectordb.searchCollection(
    "uploads",
    embeddings[0],
    4,
    {
      includeChunkContent: true,
      contentEncoding: "utf-8",
      minScore: 0.7,
    }
  );

  const pluginsSearchResults = await vectordb.searchCollection(
    "plugins",
    embeddings[0],
    5,
    {
      includeChunkContent: true,
      contentEncoding: "utf-8",
      minScore: 0.75,
    }
  );

  const aiFunctions = pluginsSearchResults.map((r) => {
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
    Buffer.from(JSON.stringify({ queryId: request.id }))
  );

  const {
    request: llmQueryRequest,
    response: llmQueryResponse,
    stream: aiResponseStream,
  } = await chatCompletion({
    userId: openAiUserId,
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
            ...documentsSearchResults,
          ],
          has_functions: aiFunctions.length > 0,
        }),
      },
      {
        role: "user",
        content: request.message,
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
          id: aiResponseId,
          timestamp: aiResponseTime.getTime(),
          metadata: {
            documents: documentsSearchResults.map((r) =>
              pick(r, "documentId", "score")
            ),
          },
        })
      );

      try {
        for await (const data of aiResponseStream) {
          if (data.json) {
            const { delta } = data.json.choices[0];
            aiResponse = mergeDelta(aiResponse, delta);
            if (delta.function_call?.name) {
              // @ts-expect-error
              aiResponse.function_call!.id = aiFunctions.find(
                (f) => f.name == delta.function_call?.name
              )!.id;
            }
            controller.enqueue(JSON.stringify({ delta }));
          }
        }

        await ctx.dbs.default.query(
          `INSERT INTO chat_messages
            (id, channel_id, parent_id, role, message, model, metadata, timestamp)
            VALUES (?,?,?,?,?,?,?,?)`,
          [
            aiResponseId,
            channelId,
            request.id,
            "ai",
            JSON.stringify(aiResponse),
            llmQueryRequest.model,
            JSON.stringify({
              documents: documentsSearchResults.map((r) =>
                omit(r, "content", "context")
              ),
            }),
            aiResponseTime.getTime(),
          ]
        );
      } catch (e) {
        controller.error(e);
      }
      controller.close();
    },
  });

  return new Response(stream, {
    status: 200,
    headers: [["content-type", "text/event-stream"]],
  });
});

const deleteMessage = p.delete(async ({ ctx, params }) => {
  await ctx.dbs.default.query(
    `DELETE FROM chat_messages where id = ? AND channel_id = ?`,
    [params.id, params.channelId]
  );
  return { success: true };
});

export { listChannels, listMessages, sendMessage, deleteMessage };
