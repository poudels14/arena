import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import terser from "@rollup/plugin-terser";
import typescript from "@rollup/plugin-typescript";
import merge from "lodash/merge.js";

const buildConfig = (overrides) => {
  return merge(
    {
      output: {
        format: "iife",
        dir: "dist",
      },
      plugins: [
        resolve({
          browser: true,
          extensions: [".js", ".jsx", ".ts", ".tsx", ".json"],
          preferBuiltins: false,
        }),
        typescript(),
        commonjs(),
        json(),
        terser(),
      ],
    },
    overrides
  );
};

export default [
  buildConfig({
    input: {
      babel: "./src/babel/index.ts",
    },
  }),
  buildConfig({
    input: {
      "wasmer-wasi": "./src/wasi/wasmer.ts",
    },
  }),
];
