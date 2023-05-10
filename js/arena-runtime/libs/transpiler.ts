const { ops, opAsync } = Arena.core;

class Transpiler {
  #rid;
  root;

  constructor(config) {
    if (!ops.op_transpiler_new) {
      throw new Error("@arena/runtime/transpiler extension not enabled");
    }
    const [rid, root] = ops.op_transpiler_new(config || {});
    this.#rid = rid;
    this.root = root;
  }

  async transpileFileAsync(filename) {
    return await opAsync(
      "op_transpiler_transpile_file_async",
      this.#rid,
      filename
    );
  }

  transpileSync(code, filename) {
    return ops.op_transpiler_transpile_sync(
      this.#rid,
      filename || "<code>",
      code
    );
  }
}

export { Transpiler };
