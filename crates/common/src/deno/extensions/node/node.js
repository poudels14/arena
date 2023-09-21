import process from "builtin://process";
import path from "builtin://path";
import { Buffer } from "builtin://buffer";

((global) => {
  const env = global.process?.env || {};
  // Note(sp): merge process.env if already set
  Object.assign(process.env, env);
  Object.assign(global, {
    process,
    path,
    Buffer,
  });
})(globalThis);
