const readFileSync = (...args: [any]) => Arena.fs.readFileSync(...args);
const writeFileSync = (...args: [any]) => Arena.fs.writeFileSync(...args);
const statSync = (...args: [any]) => Arena.fs.lstatSync(...args);

const fs = {
  readFileSync,
  writeFileSync,
  statSync,
};

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  fs,
};

export { readFileSync, writeFileSync, statSync };
export default fs;
export * as promises from "./fs_promises";
