import fileURLToPath from "file-uri-to-path";
import fileUrl from "file-url";
import parseurl from "parseurl";

const pathToFileURL = (path) => {
  const url = fileUrl(path);
  return new URL(url);
};

const parse = (url) => {
  return parseurl({ url });
};

export { fileURLToPath, pathToFileURL, parse };
