// Credit: deno

export type OSType = "windows" | "linux" | "darwin" | "freebsd" | "openbsd";

const { ops } = Arena.core;

const platform = () => ops.op_node_build_os();
const osType = () => ops.op_node_build_os() as OSType;
const isWindows = () => osType() === "windows";
const isLinux = () => osType() === "linux";
const cpus = () => [];
const homedir = () => process.cwd();
const tmpdir = () => ops.op_fs_tmpdir_sync();

export {
  platform,
  osType,
  osType as type,
  isWindows,
  isLinux,
  cpus,
  homedir,
  tmpdir,
};
