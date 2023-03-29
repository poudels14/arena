"use strict";
((global) => {
  const { ops, opAsync } = Arena.core;
  Object.assign(global.Arena, {
    fs: {
      cwdSync: ops.op_fs_cwd_sync,
      lstatSync: (file) => {
        const stat = ops.op_fs_lstat_sync(file);
        const { isFile } = stat;
        return Object.assign(stat, {
          atime: new Date(stat.atimeMs),
          mtime: new Date(stat.mtimeMs),
          ctime: new Date(stat.ctimeMs),
          birthtime: new Date(stat.birthtimeMs),
          isSymbolicLink() {
            return stat.isSymlink;
          },
          isFile() {
            return isFile;
          },
        });
      },
      realpathSync: ops.op_fs_realpath_sync,
      readdirSync: ops.op_fs_readdir_sync,
      existsSync: ops.op_fs_file_exists_sync,
      mkdirSync(dir, options = {}) {
        return ops.op_fs_mkdir_sync(dir, options.recursive || false);
      },
      readFileSync: ops.op_fs_read_file_sync,
      readFile(...args) {
        return opAsync("op_fs_read_file_async", ...args);
      },
      readToString(...args) {
        return opAsync("op_fs_read_file_string_async", ...args);
      },
      readAsJson(...args) {
        return opAsync("op_fs_read_file_as_json_async", ...args);
      },
      writeFileSync(path, data, options) {
        if (options) {
          throw new Error("options not supported yet");
        }
        ops.op_fs_write_file_sync(path, data);
      }
    },
  });
})(globalThis);
