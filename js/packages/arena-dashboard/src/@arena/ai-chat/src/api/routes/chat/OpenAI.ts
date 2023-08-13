import axios from "redaxios";
import { jsonStreamToAsyncIterator } from "@arena/sdk/utils/stream";
import { merge } from "lodash-es";

type ChatCompletionRequest = {
  model?: "gpt-3.5-turbo";
  temperature?: number;
  max_tokens?: number;
  userId: string;
  message: {
    query: string;
    system: {
      content: string;
    };
  };
};

type ChatCompletionResponse = [
  ChatCompletionRequest,
  ReturnType<typeof jsonStreamToAsyncIterator>
];

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
    "https://api.openai.com/v1/chat/completions",
    {
      user: request.userId,
      model: request.model,
      stream: true,
      messages: [
        {
          role: "system",
          content: request.message.system.content,
        },
        {
          role: "user",
          content: request.message.query,
        },
      ],
    },
    {
      responseType: "stream",
      headers: {
        Authorization: `Bearer ${process.env.OPENAI_API_KEY}`,
      },
    }
  );

  return [request, jsonStreamToAsyncIterator(res.body!)];
}

export { chatCompletion };
