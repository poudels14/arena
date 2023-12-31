class FileHandle {
  #fd;

  constructor(fd) {
    this.#fd = fd;
  }
}

class Dirent {
  #entry;
  constructor(entry) {
    this.#entry = entry;
    this.name = entry.name;
    this.isDirectory = () => {
      return this.#entry.isDirectory;
    };
    this.isFile = () => {
      return this.#entry.isFile;
    };
    this.isSymbolicLink = () => {
      return this.#entry.isSymlink;
    };
  }

  isBlockDevice() {
    return false;
  }

  isCharacterDevice() {
    return false;
  }

  isFIFO() {
    return false;
  }

  isSocket() {
    return false;
  }
}

("use strict");
((global) => {
  const { core } = Arena;
  Object.assign(global.Arena, {
    fs: {
      cwdSync: (...args) => core.ops.op_fs_cwd_sync(...args),
      accessSync(path, mode) {
        // throws error if no access
        // since op returns null, return undefined here
        return core.ops.op_fs_access_sync(path, mode || 0) || undefined;
      },
      statSync: (file) => {
        const stat = core.ops.op_fs_stat_sync(file);
        return Object.assign(stat, {
          atime: new Date(stat.atimeMs),
          mtime: new Date(stat.mtimeMs),
          ctime: new Date(stat.ctimeMs),
          birthtime: new Date(stat.birthtimeMs),
          isSymbolicLink() {
            return stat.isSymlink;
          },
          isFile() {
            return stat.isFile;
          },
          isDirectory() {
            return stat.isDirectory;
          },
        });
      },
      lstatSync: (file) => {
        const stat = core.ops.op_fs_lstat_sync(file);
        return Object.assign(stat, {
          atime: new Date(stat.atimeMs),
          mtime: new Date(stat.mtimeMs),
          ctime: new Date(stat.ctimeMs),
          birthtime: new Date(stat.birthtimeMs),
          isSymbolicLink() {
            return stat.isSymlink;
          },
          isFile() {
            return stat.isFile;
          },
          isDirectory() {
            return stat.isDirectory;
          },
        });
      },
      realpathSync: (...args) => core.ops.op_fs_realpath_sync(...args),
      openSync(path) {
        const fd = core.ops.op_fs_open_sync(path);
        return new FileHandle(fd);
      },
      closeSync(handle) {
        core.ops.op_fs_close_sync(handle.fd);
      },
      readdirSync: (path, options) => {
        const dirs = core.ops.op_fs_readdir_sync(path);
        if (options.encoding == "buffer") {
          return dirs.map((dir) => Buffer.from(dir.name));
        } else if (options.withFileTypes) {
          return dirs.map((entry) => new Dirent(entry));
        } else {
          return dirs.map((dir) => dir.name);
        }
      },
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
      async readFile(path, { encoding = "utf-8" }) {
        const data = await core.opAsync("op_fs_read_file_async", path);
        if (encoding) {
          const decoder = new TextDecoder(encoding);
          return decoder.decode(data);
        }
        return data;
      },
      async readToString(...args) {
        return await core.opAsync("op_fs_read_file_string_async", ...args);
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
