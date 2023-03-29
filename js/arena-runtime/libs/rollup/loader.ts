/**
 * This loader strips typescript types when loading. Since we need
 * typescript plugin in rollup to support typescript, instead of using
 * typescript plugin, use this loader. This loader strips Typescript
 * types from file when loading
 */
const loader = (options: { replace: any }) => {
  const { Transpiler } = Arena.BuildTools;
  const transpiler = new Transpiler({
    replace: options.replace,
  });

  return {
    name: "arena-loader",
    async load(id) {
      // TODO(sagar): all files are transpiled right now to replace
      // key/value pair in `options.replace`. If there's another
      // way to replace, we can only transpile `.ts` and `.tsx` files
      // to strip Typescript
      const { code } = await transpiler.transpileFileAsync(id);
      return code;
    },
  };
};

export { loader };
