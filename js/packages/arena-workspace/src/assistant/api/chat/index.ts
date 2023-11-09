import { merge, pick, uniqBy } from "lodash-es";
import { p } from "../procedure";
import { uniqueId } from "~/utils/uniqueId";
import { ChatChannel, ChatThread } from "./types";
import { generateAiResponse } from "./sendMessage";

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
        documents: uniqBy(metadata?.documents, (d: any) => d.documentId),
        workflow: metadata?.workflow,
      },
    };
  });
});

const sendMessage = p.mutate(async ({ ctx, params, req, errors }) => {
  const now = new Date();
  const { default: sqlite } = ctx.dbs;
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

  const [channel] = await sqlite
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

  if (!channel) {
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

  /**
   * If a workflow is active, proxy the message to the workflow
   */
  if (thread.metadata.activeWorkflow) {
    // TODO
  } else {
    return await generateAiResponse({ ctx, errors }, channel, thread, {
      id: request.id,
      content: request.message,
    });
  }
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
