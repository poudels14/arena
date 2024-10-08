declare var Arena;
const { ops, opAsync } = Arena.core;

type EmbeddingsModelOptions = {
  modelId?: string;
  revision?: string;
  usePth?: boolean;
  approximate_gelu?: boolean;
};

type TokenizeOptions = {
  truncate?: boolean;
  max_length?: number;
};

type GenerateEmbeddingsOptions = {
  // defaults to true
  normalize: boolean;
};

class EmbeddingsModel {
  #id: number;
  constructor(options: EmbeddingsModelOptions) {
    this.#id = ops.op_cloud_llm_embeddings_load_model(options);
  }

  async generateEmbeddings(
    texts: string[],
    options?: GenerateEmbeddingsOptions
  ): Promise<number[][]> {
    return await opAsync("op_cloud_llm_embeddings_generate", this.#id, texts, {
      normalize: true,
      ...(options || {}),
    });
  }

  async tokenizeText(
    text: string,
    options: TokenizeOptions
  ): Promise<{ ids: number[]; offsetMapping: number[][] }> {
    return await opAsync(
      "op_cloud_llm_embeddings_tokenize",
      this.#id,
      text,
      options
    );
  }

  close() {
    ops.op_cloud_llm_embeddings_close_model(this.#id);
  }
}

export { EmbeddingsModel };
