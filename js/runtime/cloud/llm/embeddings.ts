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
  private constructor(options: EmbeddingsModelOptions) {
    this.#id = ops.op_cloud_llm_embeddings_load_model(options);
  }

  async generateEmbeddings(
    texts: string[],
    options?: GenerateEmbeddingsOptions
  ): Promise<{ ids: number[]; offsetMapping: number[][] }> {
    return await opAsync("op_cloud_llm_embeddings_generate", this.#id, texts, {
      normalize: true,
      ...(options || {}),
    });
  }

  async tokenize(
    data: any,
    options: TokenizeOptions
  ): Promise<{ ids: number[]; offsetMapping: number[][] }> {
    const decoder = new TextDecoder();
    const text = decoder.decode(data);
    return await opAsync("op_cloud_llm_embeddings_tokenize", this.#id, text, options);
  }
}

export { EmbeddingsModel };
