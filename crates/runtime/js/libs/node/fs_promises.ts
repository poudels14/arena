const promisify =
  (param: string) =>
  (...args) =>
    new Promise((r) => r(Arena.fs[param](...args)));

const lstat = promisify("lstatSync");
const realpath = promisify("realpathSync");
const readdir = promisify("readdirSync");
const readFile = (...args: [any]) => Arena.fs.readFile(...args);
const mkdir = promisify("mkdirSync");
const writeFile = promisify("writeFileSync");

export default { lstat, realpath, readdir, readFile, mkdir, writeFile };
export { lstat, realpath, readdir, readFile, mkdir, writeFile };
