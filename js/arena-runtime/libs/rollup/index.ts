import { rollup } from "rollup";
import { babel } from "@rollup/plugin-babel";
import { resolver } from "./resolver";

const plugins = {
  babel,
  arenaResolver: resolver,
};

const build = async (options) => {
  const { plugins, ...restOptions } = options || {};
  const bundle = await rollup({
    ...restOptions,
    plugins: [...(plugins || []), resolver()],
  });
  await bundle.write(options.output);
  await bundle.close();
};

export { rollup, plugins, build };
