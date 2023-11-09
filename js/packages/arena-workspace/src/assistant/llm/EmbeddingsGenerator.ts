import { once } from "lodash-es";
import ky from "ky";
import { HFTokenizer } from "@arena/cloud/llm";
import { createDocumentSplitter } from "@arena/llm/splitter";
import type { Splitter } from "@arena/llm/splitter";

const EMBEDDINGS_MODEL = "thenlper/gte-small";
// Note: get-small support upto 512 tokens but leave some buffer :shrug:
const MAX_TOKEN_LENGTH = 400;

class DocumentEmbeddingsGenerator {
  getDocumentSplitter: () => ReturnType<typeof createDocumentSplitter>;
  constructor() {
    const getTokenizer = once(
      async () =>
        await HFTokenizer.init({
          modelName: EMBEDDINGS_MODEL,
          truncate: false,
        })
    );
    this.getDocumentSplitter = once(() => {
      return createDocumentSplitter({
        async tokenize(content) {
          const tokenizer = await getTokenizer();
          return await tokenizer.tokenize(content);
        },
        maxTokenLength: MAX_TOKEN_LENGTH,
        // TODO(sagar): use the chunk before and after since the following
        // termination nodes are used
        windowTerminationNodes: ["heading", "table", "code"],
      });
    });
  }

  async getTextEmbeddings(texts: string[]) {
    const json = await ky
      .post(
        process.env.EMBEDDINGS_API_ENDPOINT ||
          "http://localhost:9004/llm/embeddings",
        {
          json: {
            model: EMBEDDINGS_MODEL,
            sentences: texts,
          },
        }
      )
      .json<{ embeddings: number[][] }>();
    return json.embeddings;
  }

  async getChunkEmbeddings(chunks: Splitter.DocumentChunk[]) {
    const embeddings = await this.getTextEmbeddings(
      chunks.map((c) => c.content)
    ).catch((e) => {
      throw new Error("Error generating text embeddings");
    });
    return embeddings.map((vectors, idx) => {
      const { position, content, metadata } = chunks[idx];
      const { start, end } = position;
      return {
        start,
        end,
        content,
        vectors,
        metadata,
      };
    });
  }
}

export { DocumentEmbeddingsGenerator };
