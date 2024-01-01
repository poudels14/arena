function noop() {}

const { core } = Arena;
const cwd = () => Arena.fs.cwdSync();
const env = {
  TERM: "xterm-256color",
};
const on = noop;
const memoryUsage = noop;

async function* createStdin() {}

const hrtime = () => {
  const milli = performance.now();
  const sec = Math.floor(milli / 1000);
  const nano = Math.floor(milli * 1_000_000 - sec * 1_000_000_000);
  return [sec, nano];
};

const process = {
  cwd,
  env,
  get argv() {
    return core.ops.op_node_process_args();
  },
  version: "18.19.0",
  versions: {
    node: "18.19.0",
  },
  stdin: Object.assign(createStdin(), {
    isTTY: false,
    fd: 0,
    setEncoding() {},
  }),
  stdout: {
    isTTY: false,
    // default to utf8
    write(content, encoding = "utf8") {
      if (encoding != "utf8") {
        throw new Error("Writing non utf8 content to stdout not supported");
      }
      Arena.core.ops.op_fs_write_stdout_str(content);
    },
    fd: 1,
  },
  stderr: {
    isTTY: false,
    fd: 2,
  },
  hrtime: Object.assign(hrtime, {
    bigint: () => {
      const [sec, nano] = hrtime();
      return BigInt(sec) * 1_000_000_000n + BigInt(nano);
    },
  }),
  memoryUsage,
  on,
  off(...args) {
    console.log("[node/process] OFF =", args);
  },
  once(...args) {
    console.log("[node/process] ONCE =", args);
  },
  emitWarning(...args) {
    console.log("[WARNING]", ...args);
  },
};

export default process;
export { cwd, env, memoryUsage };
