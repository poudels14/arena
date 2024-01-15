type ChatContext = {
  documents: {
    content: string;
  }[];
  has_functions: boolean;
};

const generateSystemPrompt = (context: ChatContext) => {
  if (context.documents.length > 0 && context.has_functions) {
    return generatePomptWithContextAndFunctions(context.documents);
  }
  if (context.documents.length > 0) {
    return generatePomptWithContextOnly(context.documents);
  }
  return `You are an AI assistant. You should answer the question asked by the user as accurately as possible. If you aren't sure about the answer, say so.`;
};

const generatePomptWithContextOnly = (documents: ChatContext["documents"]) => {
  return `You are an AI assistant. You should answer the question asked by the user as accurately as possible using the given context. If the context doesn't have the answer to the user's question, you should say you can't find the answer to the question.
  
  Context:

  ${documents
    .map((d, idx) => `document ${idx}: \n\n ${d.content}`)
    .join("\n\n")}`;
};

const generatePomptWithContextAndFunctions = (
  documents: ChatContext["documents"]
) => {
  return `You are an AI assistant. You are provided with context related to the question being asked as well as a list of functions that can be used to perform the task that matches with what the user is asking for. You should answer the question asked by the user as accurately as possible using the context provided or use one of the functions passed to answer the user query.

  Context:
  
  ${documents
    .map((d, idx) => `document ${idx}: \n\n ${d.content}`)
    .join("\n\n")}`;
};

// TODO
// THIS IS FROM pdf.ai
('You are a helpful assistant with limited access to info designed to answer questions ONLY from the given document content else say that you don\'t know the answer and always answer the queries in the language they are asked in. If the "QUESTION" is in English, answer in English. If the "QUESTION" is in Spanish, answer in Spanish and similarly if the "QUESTION" is in XYZ language, answer it in the same XYZ language. Be as accurate as possible in providing answers only from the given document context. You are not like ChatGPT that answers every question. Answer only if it found in the given document content.');

export { generateSystemPrompt };
