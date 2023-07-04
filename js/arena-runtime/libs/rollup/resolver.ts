import path from "path";
import { Resolver } from "@arena/runtime/resolver";

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
  const resolver = new Resolver({
    preserveSymlink: true,
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
      try {
        const resolvedPath = resolver.resolve(source, importer || "./");
        if (resolvedPath) {
          return {
            id: path.join(resolver.root, resolvedPath),
            external: false,
            resolvedBy: "arena-resolver",
          };
        }
      } catch (e) {
        return null;
      }
    },
  };
};

export { resolver };
