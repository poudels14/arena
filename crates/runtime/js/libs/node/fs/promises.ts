const promisify =
  (param: string) =>
  (...args) =>
    new Promise((r) => r(Arena.fs[param](...args)));

const exists = promisify("existsSync");
const access = promisify("accessSync");
const stat = promisify("statSync");
const lstat = promisify("lstatSync");
const realpath = promisify("realpathSync");
const readdir = promisify("readdirSync");
const readFile = (...args: [any]) => Arena.fs.readFile(...args);
const mkdir = promisify("mkdirSync");
const writeFile = promisify("writeFileSync");

export {
  exists,
  access,
  stat,
  lstat,
  realpath,
  readdir,
  readFile,
  mkdir,
  writeFile,
};
