import { keyBy, merge, pick } from "lodash-es";
import z from "zod";
import mime from "mime";
import ky from "ky";
import { ReplaySubject } from "rxjs";
import { Workspace } from "@portal/workspace-sdk";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { p } from "../procedure";
import { ChatThread } from "./types";
import { generateLLMResponseStream } from "./chains/query";
import { ChatMessage } from "../repo/chatMessages";
import { ThreadOperationsStream } from "../../chatsdk";
import { generateQueryTitle } from "./chains/title";

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

  const messages = (await listMessages({
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
  })) as unknown as any[];

  const artifacts = await ctx.repo.artifacts.list(
    {
      threadId,
    },
    {
      limit: 100,
    }
  );

  return {
    ...thread,
    messages: messages.map((message) => {
      return {
        ...message,
        artifacts: artifacts
          .filter((a) => a.messageId == message.id)
          .map((artifact) => {
            return {
              ...pick(artifact, "id", "name", "metadata", "createdAt"),
              contentType: mime.getType(artifact.name),
            };
          }),
      };
    }),
  };
});

const deleteThread = p.mutate(async ({ req, ctx, params, errors }) => {
  const { threadId } = params;
  const thread = await ctx.repo.chatThreads.getById(threadId);
  if (!thread) {
    return errors.notFound();
  }

  await ctx.repo.chatMessages.deleteByThreadId({ threadId: thread.id });
  await ctx.repo.artifacts.deleteByThreadId({ threadId: thread.id });
  await ctx.repo.taskExecutions.deleteByThreadId({ threadId: thread.id });
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
      metadata: pick(m.metadata, "context", "error", "searchResults"),
    };
  });
});

const sendMessage = p
  .input(
    z.object({
      id: z.string().optional(),
      thread: z.any(),
      parentId: z.string().nullable(),
      // one of the models provided by workspace /api/llm/models
      model: z.object({
        id: z.string(),
      }),
      message: z.object({
        content: z.string(),
      }),
      // this is used for branching feature
      idFilter: z.array(z.string()).optional(),
      regenerate: z.boolean().optional(),
      temperature: z.number().optional(),
      // chat query context
      context: z.array(
        z.object({
          app: z.object({
            id: z.string(),
          }),
          breadcrumbs: z.array(
            z.object({
              id: z.string(),
              contentType: z.string().optional(),
            })
          ),
        })
      ),
    })
  )
  .mutate(async ({ ctx, params, body, errors }) => {
    const model = await getModelById(
      ctx.env.PORTAL_WORKSPACE_HOST,
      body.model.id
    );
    if (!model) {
      return errors.badRequest("Invalid model");
    }
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
          model: {
            id: model.id,
            name: model.name,
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
    const filteredOldMessages = oldMessages.filter(
      (m) => !body.idFilter || body.idFilter.includes(m.id)
    );

    const newMessage: ChatMessage = {
      id: body.id,
      message: body.message,
      threadId: thread.id,
      role: "user",
      userId: ctx.user?.id!,
      createdAt: now,
      metadata: {
        context: body.context,
      },
      parentId: body.parentId || null,
    };

    if (!body.regenerate) {
      await ctx.repo.chatMessages.insert(newMessage);
    }

    const replayStream = new ReplaySubject<any>();
    const opsStream = new ThreadOperationsStream(thread.id, replayStream);
    if (Boolean(!existingThread)) {
      opsStream.addNewThread(thread);
    }
    if (!body.regenerate) {
      opsStream.sendNewMessage(newMessage);
    }
    generateLLMResponseStream(
      { ctx, errors },
      {
        model,
        opsStream,
        thread,
        message: newMessage,
        previousMessages: filteredOldMessages,
        options: {
          temperature: body.temperature || 0.9,
          context: body.context,
        },
      }
    );

    if (!existingThread) {
      const title = await generateQueryTitle(model, body.message.content);
      await ctx.repo.chatThreads.update({
        id: thread.id,
        title,
      });
      opsStream.setThreadTitle(title);
    }

    const responseStream = new ReadableStream({
      async start(controller) {
        replayStream.subscribe({
          next(json) {
            try {
              controller.enqueue("data: " + JSON.stringify(json) + "\n");
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

let AVAILABLE_MODELS_CACHE: Workspace.Model[] = [];
const getModelById = async (workspaceHost: string, modelId: string) => {
  const model = AVAILABLE_MODELS_CACHE.find((m) => m.id == modelId);
  if (model) {
    return model;
  }

  const models = await ky
    .get(new URL(`/api/llm/models`, workspaceHost).href)
    .json<Workspace.Model[]>();
  AVAILABLE_MODELS_CACHE = models;

  return models.find((m) => m.id == modelId && !m.disabled);
};

export {
  listThreads,
  getThread,
  deleteThread,
  listMessages,
  sendMessage,
  deleteMessage,
  listActiveTasks,
};
