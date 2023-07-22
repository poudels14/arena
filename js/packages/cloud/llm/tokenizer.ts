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
  public static async init(modelName?: string) {
    const id = await opAsync(
      "op_cloud_llm_hf_new_pretrained_tokenizer",
      modelName || "sentence-transformers/all-MiniLM-L6-v2"
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
