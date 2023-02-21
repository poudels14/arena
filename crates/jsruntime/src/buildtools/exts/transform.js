"use strict";
((global) => {
  const { ops } = Arena.core;
  
  Object.assign(global.Arena.BuildTools, {
    async transformFileAsync(filename, options) {
      return await ops.op_buildtools_transform_file_async(filename, options || {});
    },
    transformSync(code, options) {
      return ops.op_buildtools_transform_sync(code, options || {});
    }
  });
})(globalThis);