import { keyBy, merge, pick } from "lodash-es";
import z from "zod";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import ky from "ky";
import { p } from "../procedure";
import { ChatThread } from "./types";
import { generateLLMResponseStream } from "./aiResponse";
import { ChatMessage } from "../repo/chatMessages";
import { Search } from "@portal/workspace-sdk/llm/search";
import { klona } from "klona";
import { dset } from "dset";

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
        searchResults: m.metadata?.searchResults,
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
      // chat query context
      context: z
        .object({
          app: z.object({
            id: z.string(),
          }),
          breadcrumbs: z.array(
            z.object({
              id: z.string(),
            })
          ),
        })
        .optional()
        .nullable(),
    })
  )
  .mutate(async ({ ctx, env, params, req, body, errors }) => {
    const now = new Date();
    // TODO(sagar): abstract several steps and use middleware like system
    // so that it's easier to build plugins for chat system itself. For example,
    // there can be a plugin to augment prompts to provide better results, or
    // a plugin to provide additional context based on the prompt, or a plugin
    // to built agent like multi-step llm querries
    body.id = body.id || uniqueId();
    body.thread = merge(
      {
        id: params.threadId,
        metadata: {
          ai: {
            // model: "gpt-3.5-turbo",
            model: "gpt-4-1106-preview",
          },
        },
      },
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

    const oldMessages = await ctx.repo.chatMessages.list({
      threadId: thread.id,
    });
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

    let searchResults = [];
    if (body.context?.app?.id) {
      const { app, breadcrumbs } = body.context!;
      // TODO: error handling
      const activeContextSearchResult = await ky
        .post(
          new URL(
            `/w/apps/${app.id}/api/portal/llm/search`,
            ctx.env.PORTAL_WORKSPACE_HOST
          ).href,
          {
            json: {
              query: body.message.content,
              context: {
                breadcrumbs,
              },
            },
          }
        )
        .json<Search.Response>();

      if (
        activeContextSearchResult.files.length > 0 ||
        activeContextSearchResult.tools.length > 0
      ) {
        searchResults.push({
          app: {
            id: app.id,
          },
          ...activeContextSearchResult,
        });
      }
    }

    if (searchResults.length > 0) {
      const clonedSearchResults = klona(searchResults);
      clonedSearchResults.forEach((result) => {
        result.files.forEach((file) => {
          file.chunks.forEach((chunk) => {
            // clear the chunk content to avoid storing duplicate data
            dset(chunk, "content", undefined);
          });
        });
      });

      await ctx.repo.chatMessages.insert({
        id: uniqueId(),
        message: {
          content: "",
        },
        threadId: thread.id,
        role: "system",
        userId: null,
        createdAt: now,
        metadata: {
          searchResults: clonedSearchResults,
        },
        parentId: newMessage.id,
      });
    }

    const stream = await generateLLMResponseStream(
      { ctx, errors },
      {
        thread,
        message: newMessage,
        previousMessages: oldMessages,
        searchResults,
        context: body.context,
      }
    );

    const responseStream = new ReadableStream({
      async start(controller) {
        if (Boolean(!existingThread)) {
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
                path: ["messages", newMessage.id],
                value: newMessage,
              },
            ],
          })
        );
        stream.subscribe({
          next(json) {
            try {
              controller.enqueue(JSON.stringify(json));
            } catch (e) {}
          },
          complete() {
            controller.close();
          },
        });
      },
    });

    return new Response(responseStream, {
      status: 200,
      headers: [["content-type", "text/event-stream"]],
    });
  });

const deleteMessage = p.delete(async ({ ctx, params }) => {
  await ctx.dbpool.query(
    `DELETE FROM chat_messages where id = ? AND channel_id = ?`,
    [params.id, params.channelId]
  );
  return { success: true };
});

const listActiveTasks = p.query(async ({ ctx, params }) => {
  const { threadId } = params;
  const taskExecutions = await ctx.repo.taskExecutions.list({ threadId });

  return keyBy(taskExecutions, (task) => task.id);
});

export {
  listThreads,
  getThread,
  deleteThread,
  listMessages,
  sendMessage,
  deleteMessage,
  listActiveTasks,
};
