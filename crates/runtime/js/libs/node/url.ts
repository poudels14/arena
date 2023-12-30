import fileURLToPath from "file-uri-to-path";
import { parse } from "url-parse";

const url = {
  fileURLToPath,
  parse,
};

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  url,
};

export default url;
export { fileURLToPath, parse };
