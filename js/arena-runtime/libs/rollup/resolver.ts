import path from "path";

/**
 * Since rollup's buitin resolver only resolves ".js" and ".mjs",
 * use custom resolver to resolve JSX and typescript files.
 *
 * This resolver also resolves node modules and don't require
 * @rollup/plugin-node-resolve
 */
const resolver = (options: Arena.ResolverConfig = {}) => {
  const { Resolver } = Arena.BuildTools;
  const resolver = new Resolver({
    preserve_symlink: true,
    ...options,
  });

  return {
    name: "arena-resolver",
    async resolveId(source, importer, options) {
      const resolvedPath = resolver.resolve(source, importer || "./");
      if (resolvedPath) {
        return {
          id: path.join(resolver.root, resolvedPath),
          external: false,
          resolvedBy: "arena-resolver",
        };
      }
    },
  };
};

export { resolver };
