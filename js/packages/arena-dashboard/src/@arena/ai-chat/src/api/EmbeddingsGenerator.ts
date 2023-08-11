import { once } from "lodash-es";
import ky from "ky";
import { HFTokenizer } from "@arena/cloud/llm/tokenizer";
import { createDocumentSplitter } from "@arena/llm/splitter";
import type { Splitter } from "@arena/llm/splitter";

// token for `.` in `all-MiniLM-L6-v2` model
// TODO(sagar): maybe use special token mask instead?
const DOT_TOKEN = 1012;
const EMBEDDINGS_MODEL = "thenlper/gte-small";
const MAX_TOKEN_LENGTH = 200;

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
        specialTokens: {
          dot: DOT_TOKEN,
        },
      });
    });
  }

  async split(document: Splitter.Document) {
    return this.getDocumentSplitter()(document);
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
    );
    return embeddings.map((vectors, idx) => {
      const { position, content } = chunks[idx];
      const { start, end } = position;
      return {
        start,
        end,
        content,
        vectors,
      };
    });
  }
}

export { DocumentEmbeddingsGenerator };
