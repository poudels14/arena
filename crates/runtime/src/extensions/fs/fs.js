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
      readFileSync: (path, encoding) => {
        const data = core.ops.op_fs_read_file_sync(path);
        if (encoding) {
          const decoder = new TextDecoder(encoding);
          return decoder.decode(data);
        }
        return data;
      },
      readFile(path, encoding) {
        const data = core.opAsync("op_fs_read_file_async", path);
        if (encoding) {
          const decoder = new TextDecoder(encoding);
          return decoder.decode(data);
        }
        return data;
      },
      readToString(...args) {
        return core.opAsync("op_fs_read_file_string_async", ...args);
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
