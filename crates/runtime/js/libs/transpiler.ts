const { ops, opAsync } = Arena.core;

// Note: keep this in sync with @portal/server-dev Transpiler
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

  async transpileFile(filename) {
    return await opAsync(
      "op_transpiler_transpile_file_async",
      this.#rid,
      filename
    );
  }

  async transpileCode(code, filename) {
    return ops.op_transpiler_transpile_sync(
      this.#rid,
      filename || "<code>",
      code
    );
  }
}

Arena.__arenaRuntime = {
  ...(Arena.__arenaRuntime || {}),
  "@arena/runtime/transpiler": { Transpiler },
};

export { Transpiler };
