class FileHandle {
  #fd;

  constructor(fd) {
    this.#fd = fd;
  }
}

class Stat {
  #stat;
  constructor(stat) {
    this.#stat = stat;
    Object.assign(this, stat);
    this.atime = new Date(stat.atimeMs);
    this.mtime = new Date(stat.mtimeMs);
    this.ctime = new Date(stat.ctimeMs);
    this.birthtime = new Date(stat.birthtimeMs);
    this.isDirectory = () => {
      return this.#stat.isDirectory;
    };
    this.isFile = () => {
      return this.#stat.isFile;
    };
    this.isSymbolicLink = () => {
      return this.#stat.isSymlink;
    };
    this.isBlockDevice = () => {
      return false;
    };
    this.isCharacterDevice = () => {
      return false;
    };
    this.isFIFO = () => {
      return false;
    };
    this.isSocket = () => {
      return false;
    };
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
    this.isBlockDevice = () => {
      return false;
    };
    this.isCharacterDevice = () => {
      return false;
    };
    this.isFIFO = () => {
      return false;
    };
    this.isSocket = () => {
      return false;
    };
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
        return new Stat(stat);
      },
      lstatSync: (file) => {
        const stat = core.ops.op_fs_lstat_sync(file);
        return new Stat(stat);
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
      readFileSync: (path, options = {}) => {
        const data = Buffer.from(core.ops.op_fs_read_file_sync(path));
        const encoding =
          typeof options == "string" ? options : options.encoding;
        if (encoding) {
          const decoder = new TextDecoder(encoding);
          return decoder.decode(data);
        }
        return data;
      },
      async readFile(path, options = {}) {
        const data = Buffer.from(
          await core.opAsync("op_fs_read_file_async", path)
        );
        const encoding =
          typeof options == "string" ? options : options.encoding;
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
