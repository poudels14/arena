import * as promises from "./fs_promises";

const callFs =
  (name) =>
  (...args) =>
    Arena.fs[name](...args);

const readFileSync = callFs("readFileSync");
const writeFileSync = callFs("writeFileSync");
const statSync = callFs("statSync");
const existsSync = callFs("existsSync");
const accessSync = callFs("accessSync");
const lstatSync = callFs("lstatSync");
const readdirSync = callFs("readdirSync");

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
  existsSync,
  accessSync,
  readFileSync,
  writeFileSync,
  statSync,
  lstatSync,
  readdirSync,
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

export {
  existsSync,
  accessSync,
  readFileSync,
  writeFileSync,
  statSync,
  lstatSync,
  readdirSync,
};
export default fs;
export { promises };
