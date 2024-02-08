import tty from "tty";
import path from "path";
import * as pyodideOriginal from "../../../../pyodide/dist/pyodide.mjs";
import "../../../../pyodide/dist/pyodide.asm.js";

declare var Arena;
let { ops } = Arena.core;

let _initialized = false;
const setCompatExtension = async () => {
  if (_initialized) {
    return;
  }
  _initialized = true;

  pyodideOriginal.setCompatExtension({
    node: {
      tty,
      path,
    },
    resolvePath(path) {
      return path;
    },
    async loadLockFile(lockFileURL) {
      const lockFile = ops.op_cloud_pyodide_load_text_file(lockFileURL);
      return JSON.parse(lockFile);
    },
    fetchBinary(path, file_sub_resource_hash) {
      const data = ops.op_cloud_pyoddide_load_binary(path);
      return {
        binary: Promise.resolve(
          new Uint8Array(data, data.byteOffset, data.byteLength)
        ),
      };
    },
  });
};

const pyodide = {
  __ARENA_CLOUD: true,
  async loadPyodide(options) {
    await setCompatExtension();
    return await pyodideOriginal.loadPyodide({
      ...options,
      packageCacheDir: "builtin://@arena/cloud/pyodide/pyodide-lock.json",
      lockFileURL: "builtin://@arena/cloud/pyodide/pyodide-lock.json",
      stdLibURL: "builtin://@arena/cloud/pyodide/python_stdlib.zip",
      indexURL: "builtin://@arena/cloud/pyodide/",
    });
  },
};

export { pyodide };
