const promisify =
  (fn: any) =>
  (...args) =>
    new Promise((r) => r(fn(...args)));

const fs = Arena.fs;
const lstat = promisify(fs.lstatSync);
const realpath = promisify(fs.realpathSync);
const readdir = promisify(fs.readdirSync);
const readFile = Arena.fs.readFile;
const mkdir = promisify(Arena.fs.mkdirSync);
const writeFile = promisify(Arena.fs.writeFileSync);

export { lstat, realpath, readdir, readFile, mkdir, writeFile };
