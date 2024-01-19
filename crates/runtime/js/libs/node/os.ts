// Credit: deno

export type OSType = "windows" | "linux" | "darwin" | "freebsd" | "openbsd";

const { ops } = Arena.core;

function arch() {
  const arch = ops.op_node_build_arch();
  if (arch == "x86_64") {
    return "x64";
  } else if (arch == "aarch64") {
    return "arm64";
  } else {
    throw Error("unreachable");
  }
}
const platform = () => ops.op_node_build_os();
const osType = () => ops.op_node_build_os() as OSType;
const isWindows = () => osType() === "windows";
const isLinux = () => osType() === "linux";
const cpus = () => [];
const homedir = () => process.cwd();
const tmpdir = () => ops.op_fs_tmpdir_sync();

export {
  arch,
  platform,
  osType,
  osType as type,
  isWindows,
  isLinux,
  cpus,
  homedir,
  tmpdir,
};
