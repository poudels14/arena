// Credit: deno

export type OSType = "windows" | "linux" | "darwin" | "freebsd" | "openbsd";

const { ops } = Arena.core;

const platform = () => ops.op_node_build_os();
const osType = () => ops.op_node_build_os() as OSType;
const isWindows = () => osType() === "windows";
const isLinux = () => osType() === "linux";
const cpus = () => [];

const os = { platform, osType, isLinux, isWindows, cpus };

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  os,
};

export default os;
export { platform, osType, isWindows, isLinux, cpus };
