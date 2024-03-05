import { ChatOpenAI } from "@langchain/openai";
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

import { Context } from "../../procedure";
import { ThreadOperationsStream } from "../../../chatsdk";
import { ChatThread } from "../types";
import { ChatMessage } from "../../repo/chatMessages";
import { AtalasChatMessageHistory } from "./history";
import { AtalasDriveSearch } from "./drive";

async function generateLLMResponseStream(
  { ctx }: Pick<ProcedureRequest<Context, any>, "ctx" | "errors">,
  {
    opsStream,
    thread,
    message,
    previousMessages,
    context,
  }: {
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
  const model = new ChatOpenAI({
    // openAIApiKey: "",
  });
  const outputParser = new StringOutputParser();

  const chain = prompt.pipe(model).pipe(outputParser);
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

  opsStream.close();
}

export { generateLLMResponseStream };
