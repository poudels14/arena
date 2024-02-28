import { Middleware } from "@portal/sdk/llm/chain";
import { Search } from "@portal/workspace-sdk/llm/search";

function createQueryContextExtension(searchResults: Search.Response[]) {
  return {
    async middleware(): Promise<Middleware> {
      return function addFunctions({ request }) {
        const files = searchResults.flatMap((result) => result.files || []);
        if (files.length > 0) {
          request.messages.push({
            role: "system",
            content:
              `Here are some files that might be related to users query. Use this information to if it is applicable, else, ignore the provided content and answer from your own knowledge.` +
              `\n` +
              files
                .map((file) => {
                  return (
                    `Filename: ${file.name}\n` +
                    "Content: " +
                    file.chunks.map((c) => c.content)
                  );
                })
                .join("\n") +
              `\n\n`,
          });
        }
      };
    },
  };
}

export { createQueryContextExtension };
