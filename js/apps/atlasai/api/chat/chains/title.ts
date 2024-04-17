import { ChatPromptTemplate } from "@langchain/core/prompts";
import { StringOutputParser } from "@langchain/core/output_parsers";
import { getLLMModel } from "./modelSelector";
import { Workspace } from "@portal/workspace-sdk";

async function generateQueryTitle(model: Workspace.Model, query: string) {
  const prompt = ChatPromptTemplate.fromMessages([
    [
      "system",
      "You are an AI called Atlas who is expert at generating a summary of any text. You should ALWAYS respond with a SHORT and HELPFUL summary without using QUOTES OR providing additional info.",
    ],
    ["human", "Summarize the text in few words: {input}"],
  ]);

  const chainModel = getLLMModel(model, {
    temperature: 0.9,
  });
  const outputParser = new StringOutputParser();
  const chain = prompt.pipe(chainModel).pipe(outputParser);
  const title = await chain.invoke({
    input: query,
  });

  // replace quotes if any
  return title.replaceAll(/(^")|("$)/g, "");
}

export { generateQueryTitle };
