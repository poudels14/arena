const { opAsync } = Arena.core;

class HFTokenizer {
  #id: number;
  private constructor(id: number) {
    this.#id = id;
  }

  public static async init(modelName: string) {
    const id = await opAsync(
      "op_cloud_llm_hf_new_pretrained_tokenizer",
      modelName
    );
    return new HFTokenizer(id);
  }

  async tokenize(text: string) {
    return await opAsync("op_cloud_llm_hf_encode", this.#id, text);
  }
}

export { HFTokenizer };
