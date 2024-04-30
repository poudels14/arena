import { ChatCompletionExecutor } from "@portal/cortex/executors/ChatCompletion";
import { ChatPromptTemplate } from "@portal/cortex/prompt";
import { MessagesPlaceholder } from "@portal/cortex/prompt";
import {
  createStreamDeltaStringSubscriber,
  parseStringResponse,
} from "@portal/cortex/plugins/response";
import { ProcedureRequest } from "@portal/server-core/router";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { Workspace } from "@portal/workspace-sdk";
import dedent from "dedent";
import { pick, get } from "lodash-es";

import { Context } from "../../procedure";
import { ThreadOperationsStream } from "../../../chatsdk";
import { ChatThread } from "../types";
import { ChatMessage } from "../../repo/chatMessages";
import {
  AtalasDrive,
  SearchResults,
  formatSearchResultsAsString,
} from "./drive";
import { buildModelProvider } from "./modelSelector";

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
  const drive = new AtalasDrive(
    ctx.env.PORTAL_WORKSPACE_HOST,
    ctx.repo,
    thread.id,
    message.id,
    options.context || []
  );

  const contextImage = await drive.fetchContextImage();
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

      {drive_search_results}`,
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

  const chainModel = buildModelProvider(model);
  const executor = new ChatCompletionExecutor({
    runnables: [prompt, chainModel, drive],
    variables: {
      chat_history: () => {
        return previousMessages.map((message) => {
          return {
            role:
              message.role == "system"
                ? "system"
                : message.role == "ai"
                ? "assistant"
                : "user",
            content: message.message.content!,
          };
        });
      },
      drive_search_results: async (ctxt) => {
        const searchResult = await ctxt.resolve<SearchResults>(
          "atlasai.drive.search",
          {
            argument: message.message.content as string,
          }
        );
        return formatSearchResultsAsString(searchResult as any);
      },
    },
  });

  const aiMessageId = uniqueId(19);
  const aiResponseTime = new Date();
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

  try {
    if (contextImage && !supportsImage) {
      throw new Error("This model doesn't support image");
    }

    const ctxt = await executor.invoke({
      variables: {
        input: message.message.content!,
        systemPrompt: options.systemPrompt,
      },
      config: {
        stream: true,
        temperature: options.temperature,
      },
      plugins: [
        createStreamDeltaStringSubscriber((chunk) => {
          opsStream.sendMessageChunk(aiMessageId, chunk);
        }),
      ],
    });
    const response = parseStringResponse(ctxt);
    const metadata = {
      searchResults: get(ctxt.state, "atlasai.drive.search"),
    };
    opsStream.sendMessageMetadata(aiMessageId, metadata);
    await ctx.repo.chatMessages.insert({
      id: aiMessageId,
      threadId: thread.id,
      parentId: message.id,
      role: "ai",
      message: {
        content: response,
      },
      userId: null,
      createdAt: aiResponseTime,
      metadata,
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
