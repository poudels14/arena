import { rollup } from "rollup";
import { babel } from "@rollup/plugin-babel";
import { createFilter } from "@rollup/pluginutils";
import postcss from "rollup-plugin-postcss";
import { terser } from "./terser";
import { resolver } from "./resolver";
import { loader } from "./loader";

const plugins = {
  babel,
  terser,
  postcss,
  arenaResolver: resolver,
  arenaLoader: loader,
};

const build = async (options) => {
  const { plugins, ...restOptions } = options || {};
  const bundle = await rollup({
    ...restOptions,
    plugins,
  });
  await bundle.write(options.output);
  await bundle.close();
};

export { rollup, plugins, createFilter, build };
