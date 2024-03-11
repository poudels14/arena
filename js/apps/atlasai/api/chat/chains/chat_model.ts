import type { BaseChatModelParams } from "@langchain/core/language_models/chat_models";
import {
  type OpenAIClient,
  type ChatOpenAICallOptions,
  type OpenAIChatInput,
  type OpenAICoreRequestOptions,
  ChatOpenAI,
} from "@langchain/openai";
import { getEnvironmentVariable } from "@langchain/core/utils/env";

type PortalAIUnsupportedArgs =
  | "frequencyPenalty"
  | "presencePenalty"
  | "logitBias"
  | "functions";

type PortalAIUnsupportedCallOptions = "functions" | "function_call";

export interface ChatPortalAICallOptions
  extends Omit<ChatOpenAICallOptions, PortalAIUnsupportedCallOptions> {
  response_format: {
    type: "json_object";
    schema: Record<string, unknown>;
  };
}

export interface ChatPortalAIInput
  extends Omit<OpenAIChatInput, "openAIApiKey" | PortalAIUnsupportedArgs>,
    BaseChatModelParams {
  portalAIbaseUrl: string;
  /**
   * The PortalAI API key to use for requests.
   * @default process.env.PORTAL_AI_API_KEY
   */
  portalAIApiKey?: string;
}

/**
 * Wrapper around PortalAI API for large language models fine-tuned for chat
 *
 * PortalAI API is compatible to the OpenAI API with some limitations. View the
 * full API ref at:
 * @link {https://docs.portal.ai/reference/chat-completions}
 *
 * To use, you should have the `PORTAL_AI_API_KEY` environment variable set.
 * @example
 * ```typescript
 * const model = new ChatPortalAI({
 *   temperature: 0.9,
 *   portalAIApiKey: process.env.PORTAL_AI_API_KEY,
 * });
 *
 * const response = await model.invoke([new HumanMessage("Hello there!")]);
 * console.log(response);
 * ```
 */
export class ChatPortalAI extends ChatOpenAI<ChatPortalAICallOptions> {
  static lc_name() {
    return "ChatPortalAI";
  }

  _llmType() {
    return "portalAI";
  }

  get lc_secrets(): { [key: string]: string } | undefined {
    return {
      portalAIApiKey: "PORTAL_AI_API_KEY",
    };
  }

  lc_serializable = true;

  constructor(
    fields?: Partial<
      Omit<OpenAIChatInput, "openAIApiKey" | PortalAIUnsupportedArgs>
    > &
      BaseChatModelParams & { portalAIbaseUrl: string; portalAIApiKey?: string }
  ) {
    const portalAIApiKey =
      fields?.portalAIApiKey || getEnvironmentVariable("PORTAL_AI_API_KEY");

    if (!portalAIApiKey) {
      throw new Error(
        `PortalAI API key not found. Please set the PORTAL_AI_API_KEY environment variable or provide the key into "portalAIApiKey"`
      );
    }

    super({
      ...fields,
      modelName: fields?.modelName,
      openAIApiKey: portalAIApiKey,
      configuration: {
        baseURL: fields?.portalAIbaseUrl,
      },
    });
  }

  toJSON() {
    const result = super.toJSON();

    if (
      "kwargs" in result &&
      typeof result.kwargs === "object" &&
      result.kwargs != null
    ) {
      delete result.kwargs.openai_api_key;
      delete result.kwargs.configuration;
    }

    return result;
  }

  async completionWithRetry(
    request: OpenAIClient.Chat.ChatCompletionCreateParamsStreaming,
    options?: OpenAICoreRequestOptions
  ): Promise<AsyncIterable<OpenAIClient.Chat.Completions.ChatCompletionChunk>>;

  async completionWithRetry(
    request: OpenAIClient.Chat.ChatCompletionCreateParamsNonStreaming,
    options?: OpenAICoreRequestOptions
  ): Promise<OpenAIClient.Chat.Completions.ChatCompletion>;

  /**
   * Calls the PortalAI API with retry logic in case of failures.
   * @param request The request to send to the PortalAI API.
   * @param options Optional configuration for the API call.
   * @returns The response from the PortalAI API.
   */
  async completionWithRetry(
    request:
      | OpenAIClient.Chat.ChatCompletionCreateParamsStreaming
      | OpenAIClient.Chat.ChatCompletionCreateParamsNonStreaming,
    options?: OpenAICoreRequestOptions
  ): Promise<
    | AsyncIterable<OpenAIClient.Chat.Completions.ChatCompletionChunk>
    | OpenAIClient.Chat.Completions.ChatCompletion
  > {
    delete request.frequency_penalty;
    delete request.presence_penalty;
    delete request.logit_bias;
    delete request.functions;

    if (request.stream === true) {
      return super.completionWithRetry(request, options);
    }

    return super.completionWithRetry(request, options);
  }
}
