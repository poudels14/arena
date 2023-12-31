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
  hrtime: {
    bigint: () => BigInt(performance.now()),
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
