const fs = Arena.fs;

const promisify =
  (param: string) =>
  (...args) =>
    new Promise((r) => r(fs[param](...args)));

const lstat = promisify("lstatSync");
const realpath = promisify("realpathSync");
const readdir = promisify("readdirSync");
const readFile = (...args: [any]) => fs.readFile(...args);
const mkdir = promisify("mkdirSync");
const writeFile = promisify("writeFileSync");

export { lstat, realpath, readdir, readFile, mkdir, writeFile };
