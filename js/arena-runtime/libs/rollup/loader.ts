import { createFilter } from "@rollup/pluginutils";
import { Transpiler } from "@arena/runtime/transpiler";

/**
 * This loader strips typescript types when loading. Since we need
 * typescript plugin in rollup to support typescript, instead of using
 * typescript plugin, use this loader. This loader strips Typescript
 * types from file when loading
 */
const loader = (options: { replace: any }) => {
  const transpiler = new Transpiler({
    replace: options.replace,
  });

  const filter = createFilter("**/*.(js|ts|jsx|tsx|mjs|cjs)");
  return {
    name: "arena-loader",
    async load(id) {
      // TODO(sagar): all files are transpiled right now to replace
      // key/value pair in `options.replace`. If there's another
      // way to replace, we can only transpile `.ts` and `.tsx` files
      // to strip Typescript
      if (!filter(id)) {
        return;
      }
      const { code } = await transpiler.transpileFileAsync(id);
      return code;
    },
  };
};

export { loader };
