import process from "builtin:///process";
import path from "builtin:///path";

((global) => {
  Object.assign(global, {
    process,
    path,
  });
})(globalThis);
