import * as promises from "./fs_promises";

const readFileSync = (...args: [any]) => Arena.fs.readFileSync(...args);
const writeFileSync = (...args: [any]) => Arena.fs.writeFileSync(...args);
const statSync = (...args: [any]) => Arena.fs.statSync(...args);

// Note: the function signature should match with nodejs's  such that
// util.promisify works
const nodeJsCompatFunction =
  (func) =>
  (...args) => {
    const cb = args.pop();
    try {
      const result = Arena.fs[func](...args);
      cb(null, result);
    } catch (e) {
      cb(e, null);
    }
  };

const lstat = nodeJsCompatFunction("lstatSync");
const stat = nodeJsCompatFunction("statSync");
const realpath = nodeJsCompatFunction("realpathSync");
const open = nodeJsCompatFunction("openSync");
const close = nodeJsCompatFunction("closeSync");
const readdir = nodeJsCompatFunction("readdirSync");
const readFile = nodeJsCompatFunction("readFileSync");
const writeFile = nodeJsCompatFunction("writeFileSync");

const fs = {
  readFileSync,
  writeFileSync,
  existsSync(...args) {
    return Arena.fs.existsSync(...args);
  },
  statSync,
  promises,
  readdir,
  lstat,
  stat,
  realpath,
  open,
  close,
  readFile,
  writeFile,
};

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  fs,
};

export { readFileSync, writeFileSync, statSync };
export default fs;
export { promises };
