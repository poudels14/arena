import { merge, pick, uniqBy } from "lodash-es";
import z from "zod";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { p } from "../procedure";
import { ChatThread } from "./types";
import { generateAiResponse } from "./aiResponse";
import { ChatMessage } from "../repo/chatMessages";

const listThreads = p.query(async ({ ctx }) => {
  const threads = await ctx.repo.chatThreads.list();
  return threads;
});

const getThread = p.query(async ({ req, ctx, params, errors, ...args }) => {
  const { threadId } = params;
  const thread = await ctx.repo.chatThreads.getById(threadId);
  if (!thread) {
    return errors.notFound();
  }

  const messages = await listMessages({
    ...args,
    req,
    ctx,
    params: {
      threadId,
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

const deleteThread = p.mutate(async ({ req, ctx, params, errors, ...args }) => {
  const { threadId } = params;
  const thread = await ctx.repo.chatThreads.getById(threadId);
  if (!thread) {
    return errors.notFound();
  }

  await ctx.repo.chatThreads.deleteById(thread.id);
  return thread;
});

const listMessages = p.query(async ({ ctx, params }) => {
  const messages = await ctx.repo.chatMessages.list({
    threadId: params.threadId,
  });
  return messages.map((m: any) => {
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
        "createdAt"
      ),
      metadata: {
        documents: uniqBy(m.metadata?.documents, (d: any) => d.documentId),
      },
    };
  });
});

const sendMessage = p
  .input(
    z.object({
      id: z.string().optional(),
      thread: z.any(),
      message: z.object({
        content: z.string(),
      }),
    })
  )
  .mutate(async ({ ctx, params, req, body, errors }) => {
    const now = new Date();
    // TODO(sagar): abstract several steps and use middleware like system
    // so that it's easier to build plugins for chat system itself. For example,
    // there can be a plugin to augment prompts to provide better results, or
    // a plugin to provide additional context based on the prompt, or a plugin
    // to built agent like multi-step llm querries
    body.id = body.id || uniqueId();
    body.thread = merge(
      { id: params.threadId, metadata: { ai: { model: "gpt-3.5-turbo" } } },
      body.thread
    );

    const existingThread = await ctx.repo.chatThreads.getById(body.thread.id!);
    const thread: ChatThread = existingThread || {
      id: body.thread.id!,
      title: "",
      blockedBy: null,
      metadata: body.thread.metadata as ChatThread["metadata"],
      createdAt: now,
    };

    if (!existingThread) {
      await ctx.repo.chatThreads.insert(thread);
    }

    const newMessage: ChatMessage = {
      id: body.id,
      message: body.message,
      threadId: thread.id,
      role: "user",
      userId: ctx.user?.id || null,
      createdAt: now,
      metadata: {},
      parentId: null,
    };
    await ctx.repo.chatMessages.insert(newMessage);
    return await generateAiResponse(
      { ctx, errors },
      {
        thread,
        isNewThread: Boolean(!existingThread),
        message: newMessage,
      }
    );
  });

const deleteMessage = p.delete(async ({ ctx, params }) => {
  await ctx.dbpool.query(
    `DELETE FROM chat_messages where id = ? AND channel_id = ?`,
    [params.id, params.channelId]
  );
  return { success: true };
});

export {
  listThreads,
  getThread,
  deleteThread,
  listMessages,
  sendMessage,
  deleteMessage,
};
