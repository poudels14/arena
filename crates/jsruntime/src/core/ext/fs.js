"use strict";
((global) => {
  const { ops, opAsync } = Arena.core;
  Object.assign(global.Arena, {
    fs: {
      existsSync(path) {
        return ops.op_file_exists_sync(path);
      },
      readFileSync(path) {
        return ops.op_read_file_sync(path);
      },
      readFile(path) {
        return opAsync("op_read_file_async", path);
      },
      readToString(path) {
        return opAsync("op_read_file_string_async", path);
      },
    },
  });
})(globalThis);
