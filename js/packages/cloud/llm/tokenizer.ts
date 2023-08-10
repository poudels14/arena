const { opAsync } = Arena.core;

class HFTokenizer {
  #id: number;
  private constructor(id: number) {
    this.#id = id;
  }

  /**
   *
   * @param modelName A tokenizer model to use
   *    If not specified, `sentence-transformers/all-MiniLM-L6-v2` is
   *    used by default
   *
   * @returns
   */
  public static async init(options?: {
    modelName?: string;
    // whether to truncate the input text after max length
    truncate?: boolean;
    // max length to tokenize. Input text beyond this length will be
    // truncated unless `truncate` is set to true
    // Defaults to model max length
    maxLength?: number;
  }) {
    const id = await opAsync(
      "op_cloud_llm_hf_new_pretrained_tokenizer",
      options?.modelName || "sentence-transformers/all-MiniLM-L6-v2",
      options || {}
    );
    return new HFTokenizer(id);
  }

  /**
   *
   * @param text A text to tokenize; can be a text or buffer
   * @returns
   */
  async tokenize(
    text: any
  ): Promise<{ ids: number[]; offsetMapping: number[][] }> {
    return await opAsync("op_cloud_llm_hf_encode", this.#id, text);
  }
}

export { HFTokenizer };
