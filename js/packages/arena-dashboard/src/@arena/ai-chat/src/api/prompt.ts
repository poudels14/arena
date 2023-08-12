type ChatContext = {
  documents: {
    content: string;
  }[];
};

const generateSystemPrompt = (context: ChatContext) => {
  if (context.documents.length > 0) {
    return generatePomptWithContext(context.documents);
  }
  return `You are an AI assistant. You should answer the question asked by the user as accurately as possible.`;
};

const generatePomptWithContext = (documents: ChatContext["documents"]) => {
  return `You are an AI assistant. You should answer the question asked by the user as accurately as possible using the given context. If the context doesn't have the answer to the user's question, you should say you can't find the answer to the question.
  
  Context:
  
  ${documents.map((d) => d.content).join("\n\n")}`;
};

export { generateSystemPrompt };
