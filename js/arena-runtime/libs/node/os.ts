// Credit: deno

export type OSType = "windows" | "linux" | "darwin" | "freebsd" | "openbsd";

// @ts-expect-error
export const osType: OSType = Arena.core.ops.op_node_build_os();
export const isWindows = osType === "windows";
export const isLinux = osType === "linux";
