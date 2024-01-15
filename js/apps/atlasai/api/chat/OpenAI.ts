import axios, { Response } from "redaxios";
import { jsonStreamToAsyncIterator } from "@portal/sdk/utils/stream";
import { merge } from "lodash-es";

type ChatCompletionRequest = {
  url?: string;
  model?: "gpt-3.5-turbo" | string;
  stream: boolean;
  temperature?: number;
  max_tokens?: number;
  userId: string;
  messages: {
    role: string;
    content: string;
  }[];
  functions?: {
    name: string;
    description: string;
    // JSON schema parameters definition
    parameters: any;
  }[];
};

type ChatCompletionResponse = {
  request: ChatCompletionRequest;
  response: Response<any>;
  /**
   * null if request.stream is false
   */
  stream: ReturnType<typeof jsonStreamToAsyncIterator> | null;
};

namespace OpenAIChat {
  export type StreamResponseDelta = {
    role?: "assistant";
    content?: string | null;
    function_call?: { name: string; arguments: string } | null;
  };

  export type StreamResponse = {
    id: string;
    object: "chat.completion.chunk";
    created: number;
    model: string;
    choices: {
      index: number;
      delta: StreamResponseDelta;
      finish_reason: "function_call" | "stop" | null;
    }[];
  };
}

async function chatCompletion(
  request: ChatCompletionRequest
): Promise<ChatCompletionResponse> {
  request = merge(
    {
      model: "gpt-3.5-turbo",
    },
    request
  );

  const res = await axios.post(
    request.url || "https://api.openai.com/v1/chat/completions",
    {
      user: request.userId,
      model: request.model,
      stream: request.stream,
      messages: request.messages,
      functions: request.functions,
    },
    {
      responseType: request.stream ? "stream" : "json",
      headers: {
        Authorization: `Bearer ${process.env.OPENAI_API_KEY}`,
      },
    }
  );

  return {
    request,
    response: res,
    stream: request.stream ? jsonStreamToAsyncIterator(res.body!) : null,
  };
}

export { chatCompletion };
export type { OpenAIChat };
