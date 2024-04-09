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
import { ProcedureRequest } from "@portal/server-core/router";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { Workspace } from "@portal/workspace-sdk";
import dedent from "dedent";
import { pick } from "lodash-es";

import { Context } from "../../procedure";
import { ThreadOperationsStream } from "../../../chatsdk";
import { ChatThread } from "../types";
import { ChatMessage } from "../../repo/chatMessages";
import { AtalasChatMessageHistory } from "./history";
import { AtalasDrive, formatDocumentsAsString } from "./drive";
import { getLLMModel } from "./modelSelector";

async function generateLLMResponseStream(
  { ctx }: Pick<ProcedureRequest<Context, any>, "ctx" | "errors">,
  {
    model,
    opsStream,
    thread,
    message,
    previousMessages,
    options,
  }: {
    model: Workspace.Model;

    opsStream: ThreadOperationsStream;
    thread: ChatThread;
    message: ChatMessage;
    // old messages in the thread
    previousMessages: ChatMessage[];
    options: {
      temperature: number;
      systemPrompt: string;
      context?: {
        app: {
          id: string;
        };
        breadcrumbs: {
          id: string;
        }[];
      }[];
      selectedChatProfileId?: string;
    };
  }
) {
  const driveSearch = new AtalasDrive(
    ctx.env.PORTAL_WORKSPACE_HOST,
    ctx.repo,
    thread.id,
    message.id,
    options.context || []
  );

  const contextImage = await driveSearch.fetchContextImage();
  if (contextImage) {
    const artifact = {
      id: uniqueId(23),
      threadId: thread.id,
      messageId: message.id,
      createdAt: new Date(),
      name: contextImage.name,
      file: contextImage.file,
      size: contextImage.size || 0,
      metadata: {},
    };
    await ctx.repo.artifacts.insert(artifact);
    opsStream.sendMessageArtifact(
      message.id,
      pick(artifact, "id", "name", "createdAt", "metadata")
    );
  }

  const supportsImage = model.modalities.includes("image");
  const prompt = ChatPromptTemplate.fromMessages([
    [
      "system",
      dedent`
      {systemPrompt}


      {context}`,
    ],
    new MessagesPlaceholder("chat_history"),
    [
      "human",
      contextImage
        ? [
            {
              type: "text",
              text: "{input}",
            },
            {
              type: "image_url",
              image_url: `data:${contextImage.contentType};base64,${contextImage.file.content}`,
            },
          ]
        : "{input}",
    ],
  ]);

  const chainModel = getLLMModel(model, {
    temperature: options.temperature,
  });
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
      systemPrompt: () => options.systemPrompt,
      input: new RunnablePassthrough(),
    },
    chainWithHistory,
  ]);

  try {
    if (contextImage && !supportsImage) {
      throw new Error("This model doesn't support image");
    }

    const stream = await chatWithDocuments.stream(message.message.content!, {
      configurable: { sessionId: "sessionId" },
      // callbacks: [new ConsoleCallbackHandler()],
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
    console.log(e);
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
