import { Workspace } from "@portal/workspace-sdk";
import { ChatCompletionExecutor } from "@portal/cortex/executors/ChatCompletion";
import { ChatPromptTemplate } from "@portal/cortex/prompt";
import { parseStringResponse } from "@portal/cortex/plugins/response";

import { buildModelProvider } from "./modelSelector";

async function generateQueryTitle(modelInfo: Workspace.Model, query: string) {
  const prompt = ChatPromptTemplate.fromMessages([
    [
      "system",
      "You are an AI called Atlas who is expert at generating a title for any text. You should ALWAYS respond with a SHORT and HELPFUL summary/title without using QUOTES OR providing additional info.",
    ],
    ["human", "Generate a title for the following text:\n{query}"],
  ]);

  const model = buildModelProvider(modelInfo);
  const executor = new ChatCompletionExecutor({
    runnables: [prompt, model],
  });

  const title = await executor
    .invoke({
      variables: {
        query,
      },
      config: {
        temperature: 0.5,
      },
    })
    .then(parseStringResponse);
  return title;
}

export { generateQueryTitle };
