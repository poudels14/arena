import { ChatOpenAI } from "@langchain/openai";
import { ChatPromptTemplate } from "@langchain/core/prompts";
import { StringOutputParser } from "@langchain/core/output_parsers";

async function generateQueryTitle(query: string) {
  const prompt = ChatPromptTemplate.fromMessages([
    [
      "human",
      "You are an expert at generating title of a user query. Don't use quotes. Keep it short and simple. Generate a title for follow query: {query}",
    ],
  ]);
  const model = new ChatOpenAI({
    // openAIApiKey: "",
  });
  const outputParser = new StringOutputParser();

  const chain = prompt.pipe(model).pipe(outputParser);
  const title = await chain.invoke({
    query,
  });

  // replace quotes if any
  return title.replaceAll(/(^")|("$)/g, "");
}

export { generateQueryTitle };
