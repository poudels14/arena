import { join } from "path";
import { declare } from "@babel/helper-plugin-utils";
import * as t from "@babel/types";
import { Resolver } from "@arena/runtime/resolver";

const resolver = declare((_api, options) => {
  const state = {
    // TODO(sagar): close resolver
    resolver: new Resolver({
      preserveSymlink: true,
      ...options,
    }),
  };

  return {
    name: "babel-plugin-arena-resolve-import",
    visitor: {
      ImportDeclaration: {
        enter(path: any) {
          // Note(sagar): since all import/exports are resolved
          // even before the code is passed through babel, only
          // resolve a path that doesn't start with "./" or "/"
          const src = path.node.source.value;
          if (!src.startsWith("./") && !src.startsWith("/")) {
            const resolvedImport = state.resolver.resolve(
              path.node.source.value,
              "./" // TODO(sagar): support customizing referer?
            );
            path.node.source = t.stringLiteral(
              // Note(sagar): since resolved path is relative to project root,
              // convert it to absolute path. This is necessary since this
              // plugin is used to resolve import in dev mode for browser files
              // and all assets are served from project root
              join("/", resolvedImport)
            );
          }
        },
      },
    },
  };
});

export default resolver;
