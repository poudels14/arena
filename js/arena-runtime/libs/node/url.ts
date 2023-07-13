// Credit: deno
import { isWindows, osType } from "./os";

const CHAR_LOWERCASE_A = 97; /* a */
const CHAR_LOWERCASE_Z = 122; /* z */
const forwardSlashRegEx = /\//g;

/**
 * This function ensures the correct decodings of percent-encoded characters as well as ensuring a cross-platform valid absolute path string.
 * @see Tested in `parallel/test-fileurltopath.js`.
 * @param path The file URL string or URL object to convert to a path.
 * @returns The fully-resolved platform-specific Node.js file path.
 */
function fileURLToPath(path: string | URL): string {
  if (typeof path === "string") path = new URL(path);
  else if (!(path instanceof URL)) {
    throw new Error("Expected path to be either 'string' or 'URL'");
  }
  if (path.protocol !== "file:") {
    throw new Error("Expected URL scheme to be 'file'");
  }
  return isWindows ? getPathFromURLWin(path) : getPathFromURLPosix(path);
}

function getPathFromURLWin(url: URL): string {
  const hostname = url.hostname;
  let pathname = url.pathname;
  for (let n = 0; n < pathname.length; n++) {
    if (pathname[n] === "%") {
      const third = pathname.codePointAt(n + 2)! | 0x20;
      if (
        (pathname[n + 1] === "2" && third === 102) || // 2f 2F /
        (pathname[n + 1] === "5" && third === 99) // 5c 5C \
      ) {
        throw invalidFileUrlPathError(
          "must not include encoded \\ or / characters"
        );
      }
    }
  }

  pathname = pathname.replace(forwardSlashRegEx, "\\");
  pathname = decodeURIComponent(pathname);
  if (hostname !== "") {
    // TODO(bartlomieju): add support for punycode encodings
    return `\\\\${hostname}${pathname}`;
  } else {
    // Otherwise, it's a local path that requires a drive letter
    const letter = pathname.codePointAt(1)! | 0x20;
    const sep = pathname[2];
    if (
      letter < CHAR_LOWERCASE_A ||
      letter > CHAR_LOWERCASE_Z || // a..z A..Z
      sep !== ":"
    ) {
      throw invalidFileUrlPathError("must be absolute");
    }
    return pathname.slice(1);
  }
}

function getPathFromURLPosix(url: URL): string {
  if (url.hostname !== "") {
    throw new Error(`File URL host must be "localhost" or empty on ${osType}`);
  }
  const pathname = url.pathname;
  for (let n = 0; n < pathname.length; n++) {
    if (pathname[n] === "%") {
      const third = pathname.codePointAt(n + 2)! | 0x20;
      if (pathname[n + 1] === "2" && third === 102) {
        throw invalidFileUrlPathError("must not include encoded / characters");
      }
    }
  }
  return decodeURIComponent(pathname);
}

const invalidFileUrlPathError = (msgSuffix: string) => {
  throw new Error(`File URL path ${msgSuffix}`);
};

export { fileURLToPath };
