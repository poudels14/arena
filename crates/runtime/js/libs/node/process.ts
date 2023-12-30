function noop() {}

const cwd = () => Arena.fs.cwdSync();
const env = {
  TERM: "xterm-256color",
};
const on = noop;
const memoryUsage = noop;

const process = {
  cwd,
  env,
  argv: [],
  versions: {
    node: "18.19.0",
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
