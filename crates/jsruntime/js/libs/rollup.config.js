import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import terser from "@rollup/plugin-terser";
import typescript from "@rollup/plugin-typescript";

export default {
  input: {
    babel: "./src/babel/index.ts",
  },
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
};
