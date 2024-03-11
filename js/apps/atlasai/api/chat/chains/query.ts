import {
  ChatPromptTemplate,
  MessagesPlaceholder,
} from "@langchain/core/prompts";
import {
  RunnablePassthrough,
  RunnableSequence,
  RunnableWithMessageHistory,
} from "@langchain/core/runnables";
import { ConsoleCallbackHandler } from "@langchain/core/tracers/console";
import { StringOutputParser } from "@langchain/core/output_parsers";
import { formatDocumentsAsString } from "langchain/util/document";
import { ProcedureRequest } from "@portal/server-core/router";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { Workspace } from "@portal/workspace-sdk";

import { Context } from "../../procedure";
import { ThreadOperationsStream } from "../../../chatsdk";
import { ChatThread } from "../types";
import { ChatMessage } from "../../repo/chatMessages";
import { AtalasChatMessageHistory } from "./history";
import { AtalasDriveSearch } from "./drive";
import { getLLMModel } from "./modelSelector";

async function generateLLMResponseStream(
  { ctx }: Pick<ProcedureRequest<Context, any>, "ctx" | "errors">,
  {
    model,
    opsStream,
    thread,
    message,
    previousMessages,
    context,
  }: {
    model: Workspace.Model;
    opsStream: ThreadOperationsStream;
    thread: ChatThread;
    message: ChatMessage;
    // old messages in the thread
    previousMessages: ChatMessage[];
    context?: {
      app: {
        id: string;
      };
      breadcrumbs: {
        id: string;
      }[];
    }[];
  }
) {
  const driveSearch = new AtalasDriveSearch(
    ctx.env.PORTAL_WORKSPACE_HOST,
    ctx.repo,
    thread.id,
    context || []
  );

  const prompt = ChatPromptTemplate.fromMessages([
    [
      "system",
      "You are a helpful assistant. Answer all questions to the best of your ability.",
    ],
    // TODO: anthropic fails with seconds message since it expects all system messages to
    // be in the first message
    [
      "system",
      `Use the following pieces of context to answer the question at the end.
    If you don't know the answer, just say that you don't know, don't try to make up an answer.
    ----------------
    {context}`,
    ],
    new MessagesPlaceholder("chat_history"),
    ["human", "{input}"],
  ]);

  const chainModel = getLLMModel(model);
  const outputParser = new StringOutputParser();
  const chain = prompt.pipe(chainModel).pipe(outputParser);
  const chainWithHistory = new RunnableWithMessageHistory({
    runnable: chain,
    inputMessagesKey: "input",
    historyMessagesKey: "chat_history",
    getMessageHistory: async (sessionId) => {
      return new AtalasChatMessageHistory(previousMessages);
    },
  });

  const chatWithDocuments = RunnableSequence.from([
    {
      context: driveSearch.pipe(formatDocumentsAsString),
      input: new RunnablePassthrough(),
    },
    chainWithHistory,
  ]);

  try {
    const stream = await chatWithDocuments.stream(message.message.content!, {
      configurable: { sessionId: "sessionId" },
      callbacks: [new ConsoleCallbackHandler()],
    });

    const aiResponseTime = new Date();
    const aiMessageId = uniqueId(19);
    opsStream.sendNewMessage({
      id: aiMessageId,
      threadId: thread.id,
      parentId: message.id,
      role: "ai",
      message: {},
      userId: null,
      createdAt: aiResponseTime,
      metadata: {},
    });

    let allChunk = "";
    for await (const chunk of stream) {
      allChunk += chunk;
      opsStream.sendMessageChunk(aiMessageId, chunk);
    }

    await ctx.repo.chatMessages.insert({
      id: aiMessageId,
      threadId: thread.id,
      parentId: message.id,
      role: "ai",
      message: {
        content: allChunk,
      },
      userId: null,
      createdAt: aiResponseTime,
      metadata: {},
    });
  } catch (e: any) {
    const errorMessage = {
      id: uniqueId(19),
      threadId: thread.id,
      parentId: message.id,
      role: "system",
      message: {
        content: "",
      },
      userId: null,
      createdAt: new Date(),
      metadata: {
        error: e.message ? e.message : e,
      },
    };
    opsStream.sendNewMessage(errorMessage);
    await ctx.repo.chatMessages.insert(errorMessage);
  } finally {
    opsStream.close();
  }
}

export { generateLLMResponseStream };
