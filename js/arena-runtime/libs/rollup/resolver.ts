import path from "path";

/**
 * Since rollup's buitin resolver only resolves ".js" and ".mjs",
 * use custom resolver to resolve JSX and typescript files.
 *
 * This resolver also resolves node modules and don't require
 * @rollup/plugin-node-resolve
 */
const resolver = (
  options: Arena.ResolverConfig & { external?: string[] } = {}
) => {
  const { Resolver } = Arena.BuildTools;
  const resolver = new Resolver({
    preserve_symlink: true,
    ...options,
  });

  return {
    name: "arena-resolver",
    async resolveId(source, importer, _options) {
      if (options.external?.includes(source)) {
        return {
          id: source,
          external: true,
          resolvedBy: "arena-resolver",
        };
      }
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
