const readFileSync = (...args: [any]) => Arena.fs.readFileSync(...args);
const statSync = (...args: [any]) => Arena.fs.lstatSync(...args);

const fs = {
  readFileSync,
  statSync,
};

export { readFileSync, statSync };
export default fs;
export * as promises from "./fs_promises";
