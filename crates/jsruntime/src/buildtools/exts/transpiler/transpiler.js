"use strict";
((global) => {
  const { ops } = Arena.core;

  class Transpiler {
    #rid;

    constructor(config) {
      const rid = ops.op_transpiler_new(config || {});
      this.#rid = rid;
    }

    async transpileFileAsync(filename) {
      return await ops.op_transpiler_transpile_file_async(this.#rid, filename);
    }

    transpileSync(code) {
      return ops.op_transpiler_transpile_sync(this.#rid, code);
    }
  }

  Object.assign(global.Arena.BuildTools, {
    Transpiler,
  });
})(globalThis);