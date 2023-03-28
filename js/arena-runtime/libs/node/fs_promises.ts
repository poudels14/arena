const promisify =
  (fn: any) =>
  (...args) =>
    new Promise((r) => r(fn(...args)));

const fs = Arena.fs;
const lstat = promisify(fs.lstat);
const realpath = promisify(fs.realpath);
const readdir = promisify(fs.readdir);
const readFile = Arena.fs.readFile;
const mkdir = promisify(Arena.fs.mkdir);
const writeFile = promisify(Arena.fs.writeFileSync);

export { lstat, realpath, readdir, readFile, mkdir, writeFile };
