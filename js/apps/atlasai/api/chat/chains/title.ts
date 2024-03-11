import { ChatPromptTemplate } from "@langchain/core/prompts";
import { StringOutputParser } from "@langchain/core/output_parsers";
import { getLLMModel } from "./modelSelector";
import { Workspace } from "@portal/workspace-sdk";

async function generateQueryTitle(model: Workspace.Model, query: string) {
  const prompt = ChatPromptTemplate.fromMessages([
    [
      "human",
      "You are an expert at generating title of a user query. Don't use quotes. Keep it short and simple. Generate a title for follow query without any explanation: {query}",
    ],
  ]);

  const chainModel = getLLMModel(model);
  const outputParser = new StringOutputParser();
  const chain = prompt.pipe(chainModel).pipe(outputParser);
  const title = await chain.invoke({
    query,
  });

  // replace quotes if any
  return title.replaceAll(/(^")|("$)/g, "");
}

export { generateQueryTitle };
