const { readFileSync, lstatSync: statSync } = Arena.fs;

const fs = {
  readFileSync,
  statSync,
};

export { readFileSync, statSync };
export default fs;
export * as promises from "./fs_promises";
