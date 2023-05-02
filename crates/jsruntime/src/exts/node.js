import process from "builtin:///process";
import path from "builtin:///path";
import { Buffer } from "builtin:///buffer";

((global) => {
  Object.assign(global, {
    process,
    path,
    Buffer,
  });
})(globalThis);
