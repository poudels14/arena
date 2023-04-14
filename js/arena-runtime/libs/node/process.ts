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
  on,
  memoryUsage,
};

export default process;
export { cwd, env, memoryUsage };
