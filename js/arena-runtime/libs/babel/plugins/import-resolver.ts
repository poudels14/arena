import { declare } from "@babel/helper-plugin-utils";
import * as t from "@babel/types";

const resolver = declare((_api, _options) => {
  const { Resolver } = Arena.BuildTools;
  const state = {
    // TODO(sagar): close resolver
    resolver: new Resolver(),
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
            path.node.source = t.stringLiteral(resolvedImport);
          }
        },
      },
    },
  };
});

export default resolver;
