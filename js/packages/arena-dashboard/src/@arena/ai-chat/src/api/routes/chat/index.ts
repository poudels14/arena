import { omit, pick } from "lodash-es";
import { p } from "../../procedure";
import { chatCompletion } from "./OpenAI";
import { generateSystemPrompt } from "./prompt";
import { DocumentEmbeddingsGenerator } from "../../EmbeddingsGenerator";
import { uniqueId } from "../../utils";

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
        "message",
        "model",
        "timestamp"
      ),
      metadata: {
        documents: metadata?.documents.map((d: any) =>
          pick(d, "documentId", "score")
        ),
      },
    };
  });
});

const sendMessage = p.mutate(async ({ ctx, params, req, errors }) => {
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
      request.message,
      new Date().getTime(),
    ]
  );

  // TODO(sagar)
  const generator = new DocumentEmbeddingsGenerator();
  const embeddings = await generator.getTextEmbeddings([request.message]);
  const vectorSearchResult = await ctx.dbs.vectordb.searchCollection(
    "uploads",
    embeddings[0],
    4,
    {
      includeChunkContent: true,
      contentEncoding: "utf-8",
      minScore: 0.7,
    }
  );

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
    message: {
      system: {
        content: generateSystemPrompt({
          // TODO(sagar): aggregate all chunks of the same document
          // into one section in the prompt
          documents: vectorSearchResult,
        }),
      },
      query: request.message,
    },
  });

  if (llmQueryResponse.status !== 200) {
    return errors.internalServerError("Error connection to the AI model");
  }

  let aiResponse = "";
  const stream = new ReadableStream({
    async start(controller) {
      controller.enqueue(
        JSON.stringify({
          id: aiResponseId,
          timestamp: aiResponseTime.getTime(),
          metadata: {
            documents: vectorSearchResult.map((r) =>
              pick(r, "documentId", "score")
            ),
          },
        })
      );
      try {
        for await (const data of aiResponseStream) {
          if (data.json) {
            const { content } = data.json.choices[0].delta;
            if (content) {
              controller.enqueue(
                JSON.stringify({
                  text: content,
                })
              );
              aiResponse += content;
            }
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
            aiResponse,
            llmQueryRequest.model,
            JSON.stringify({
              documents: vectorSearchResult.map((r) =>
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
