function noop() {}

const { core } = Arena;
const cwd = () => Arena.fs.cwdSync();
const env = {
  TERM: "xterm-256color",
};
const on = noop;
const memoryUsage = noop;

const process = {
  cwd,
  env,
  get argv() {
    return core.ops.op_node_process_args();
  },
  versions: {
    node: "18.19.0",
  },
  stdout: {
    isTTY: false,
    write(...args) {
      console.log(...args);
    },
  },
  stderr: {
    isTTY: false,
    fd: 2,
  },
  hrtime: {
    bigint: () => {
      const milli = performance.now();
      const sec = Math.floor(milli / 1000);
      const nano = Math.floor(milli * 1_000_000 - sec * 1_000_000_000);
      return BigInt(sec) * 1_000_000_000n + BigInt(nano);
    },
  },
  on,
  memoryUsage,
  off(...args) {
    console.log("[node/process] OFF =", args);
  },
  once(...args) {
    console.log("[node/process] ONCE =", args);
  },
};

export default process;
export { cwd, env, memoryUsage };
