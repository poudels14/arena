"use strict";
((global) => {
  const { core } = Arena;
  Object.assign(global.Arena, {
    fs: {
      cwdSync: (...args) => core.ops.op_fs_cwd_sync(...args),
      lstatSync: (file) => {
        const stat = core.ops.op_fs_lstat_sync(file);
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
      realpathSync: (...args) => core.ops.op_fs_realpath_sync(...args),
      readdirSync: (...args) => core.ops.op_fs_readdir_sync(...args),
      existsSync: (...args) => core.ops.op_fs_file_exists_sync(...args),
      mkdirSync(dir, options = {}) {
        return core.ops.op_fs_mkdir_sync(dir, options.recursive || false);
      },
      readFileSync: (...args) => core.ops.op_fs_read_file_sync(...args),
      readFile(...args) {
        return core.opAsync("op_fs_read_file_async", ...args);
      },
      readToString(...args) {
        return core.opAsync("op_fs_read_file_string_async", ...args);
      },
      readAsJson(...args) {
        return core.opAsync("op_fs_read_file_as_json_async", ...args);
      },
      writeFileSync(path, data, options) {
        if (options) {
          throw new Error("options not supported yet");
        }
        core.ops.op_fs_write_file_sync(path, data);
      },
    },
  });
})(globalThis);
