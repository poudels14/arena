"use strict";
((global) => {
  const { ops } = Arena.core;
  
  Object.assign(global.Arena.BuildTools, {
    Transpiler: {
      async transpileFileAsync(filename, options) {
        return await ops.op_transpiler_transpile_file_async(filename, options || {});
      },
      transpileSync(code, options) {
        return ops.op_transpiler_transpile_sync(code, options || {});
      }
    }
  });
})(globalThis);