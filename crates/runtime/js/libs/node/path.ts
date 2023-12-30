import { default as path } from "path";

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  path,
};

export default path;
export {
  basename,
  extname,
  dirname,
  relative,
  isAbsolute,
  join,
  resolve,
  posix,
  win32,
  sep,
} from "path";
