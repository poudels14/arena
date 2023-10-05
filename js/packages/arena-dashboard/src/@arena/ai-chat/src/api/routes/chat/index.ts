import { merge, omit, pick, snakeCase, uniqBy } from "lodash-es";
import { p } from "../../procedure";
import { OpenAIChat, chatCompletion } from "./OpenAI";
import { generateSystemPrompt } from "./prompt";
import { DocumentEmbeddingsGenerator } from "../../EmbeddingsGenerator";
import { uniqueId } from "../../utils";
import { mergeDelta } from "./llm";

type ChatChannel = {
  id: string;
  name: string;
  metadata: {
    enableAI: boolean;
  };
};

type ChatThread = {
  id: string;
  title: string;
  metadata: {
    ai: {
      model: string;
    };
  };
};

const listChannels = p.query(async ({ ctx }) => {
  const { rows } = await ctx.dbs.default.query(`SELECT * FROM chat_channels`);

  return rows.map((row: any) => {
    return merge(row, {
      metadata: JSON.parse(row.metadata),
    });
  });
});

const getChannel = p.query(async ({ ctx, params, errors, ...args }) => {
  const channel = await ctx.dbs.default
    .query<any>(`SELECT * FROM chat_channels WHERE id = ?`, [params.channelId])
    .then(({ rows }) => {
      const row = rows[0]!;
      return (
        row &&
        merge(row, {
          metadata: JSON.parse(row.metadata),
        })
      );
    });

  if (!channel) {
    return errors.notFound();
  }

  const threads = await listThreads({
    ...args,
    errors,
    ctx,
    params,
  });

  const messages = await listMessages({
    ...args,
    errors,
    ctx,
    params,
  });

  return {
    ...channel,
    threads,
    messages,
  };
});

const listThreads = p.query(async ({ ctx, params }) => {
  const channelId = params.channelId;
  const { rows } = await ctx.dbs.default.query(
    `SELECT * FROM chat_threads WHERE channel_id = ? ORDER BY timestamp`,
    [channelId]
  );

  return rows.map((row: any) => {
    return merge(row, {
      metadata: JSON.parse(row.metadata),
    });
  });
});

const getThread = p.query(async ({ req, ctx, params, errors, ...args }) => {
  const { channelId, threadId } = params;
  const thread = await ctx.dbs.default
    .query<any>(`SELECT * FROM chat_threads WHERE id = ? AND channel_id = ?`, [
      threadId,
      channelId,
    ])
    .then(({ rows }) => {
      if (rows[0]) {
        return merge(rows[0], {
          metadata: JSON.parse(rows[0].metadata),
        });
      }
    });
  if (!thread) {
    return errors.notFound();
  }

  const messages = await listMessages({
    ...args,
    req,
    ctx,
    params: {
      channelId,
    },
    searchParams: {
      threadId,
    },
    errors,
  });

  return {
    ...thread,
    messages,
  };
});

const listMessages = p.query(async ({ ctx, params, searchParams }) => {
  const { rows: messages } = await ctx.dbs.default.query(
    `SELECT * FROM chat_messages where channel_id = ? AND thread_id = ? ORDER BY timestamp`,
    [params.channelId, searchParams.threadId || null]
  );
  return messages.map((m: any) => {
    const metadata = JSON.parse(m.metadata);
    return {
      ...pick(
        m,
        "id",
        "channelId",
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
  const now = new Date();
  const { default: sqlite, vectordb } = ctx.dbs;
  let request: {
    id: string;
    thread: Partial<ChatThread>;
    message: string;
  };
  try {
    request = await req.json();
  } catch (e) {
    return "Error parsing request body";
  }

  const channelId = params.channelId;
  if (!request.message) {
    errors.badRequest({ error: "Message can't be empty" });
  }

  // TODO(sagar): abstract several steps and use middleware like system
  // so that it's easier to build plugins for chat system itself. For example,
  // there can be a plugin to augment prompts to provide better results, or
  // a plugin to provide additional context based on the prompt, or a plugin
  // to built agent like multi-step llm querries
  request.id = request.id || uniqueId();
  request.thread = request.thread || { id: uniqueId() };

  const channels = await sqlite
    .query<any>(`SELECT * FROM chat_channels WHERE id = ?`, [channelId])
    .then(
      ({ rows }) =>
        rows.map((row) => {
          return {
            id: row.id,
            name: row.name,
            metadata: JSON.parse(row.metadata),
          };
        }) as ChatChannel[]
    );

  const channel = channels[0];
  if (channels.length == 0) {
    return errors.badRequest({ error: "Channel doesn't exist" });
  } else if (!channel.metadata.enableAI) {
    return errors.internalServerError({ error: "Unsupported chat feature" });
  }

  const threads = await sqlite
    .query<any>(`SELECT * FROM chat_threads WHERE id = ? AND channel_id = ?`, [
      channel.id,
      request.thread.id,
    ])
    .then(
      ({ rows }) =>
        rows.map((row) => {
          return {
            id: row.id,
            title: row.title,
            metadata: JSON.parse(row.metadata),
          };
        }) as ChatThread[]
    );

  const thread = threads[0] || {
    ...request.thread,
    metadata: {
      ai: {
        model: request.thread.metadata || "gpt-3.5-turbo",
      },
    },
  };

  if (threads.length == 0) {
    await sqlite.query(
      `INSERT INTO chat_threads(id, channel_id, title, metadata, timestamp)
      VALUES (?,?,?,?,?)`,
      [
        thread.id,
        channel.id,
        thread.title || "Untitiled",
        JSON.stringify(thread.metadata),
        now.getTime(),
      ]
    );
  }

  const threadMessages = await sqlite.query(
    `SELECT * FROM chat_messages WHERE thread_id = ?`,
    [request.thread.id]
  );

  await sqlite.query(
    `INSERT INTO chat_messages(id, channel_id, thread_id, role, message, timestamp)
    VALUES (?,?,?,?,?,?)`,
    [
      request.id,
      channelId,
      request.thread.id,
      ctx.user?.id || "user",
      JSON.stringify({
        content: request.message,
      }),
      now.getTime(),
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
    model: thread.metadata.ai.model,
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

        await sqlite.query(
          `INSERT INTO chat_messages
            (id, channel_id, thread_id, parent_id, role, message, model, metadata, timestamp)
            VALUES (?,?,?,?,?,?,?,?,?)`,
          [
            aiResponseId,
            channelId,
            request.thread.id,
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

export {
  listChannels,
  getChannel,
  listThreads,
  getThread,
  listMessages,
  sendMessage,
  deleteMessage,
};
