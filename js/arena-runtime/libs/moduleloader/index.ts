import { getType } from "mime";
// @ts-expect-error
import path from "node:path";
import { pick, merge } from "lodash-es";
import { Resolver } from "@arena/runtime/resolver";
import { Transpiler } from "./transpiler";

type ModuleLoaderOptions = {
  env?: Record<string, string>;
} & Arena.ResolverConfig;

/**
 * This creates an ESM module loader that returns transpiled JS/TS
 * files given the file path
 */
const createModuleLoader = (options: ModuleLoaderOptions) => {
  const env = merge(
    Arena.env,
    // @ts-ignore
    Arena.Workspace?.config?.client?.env,
    options.env
  );

  const resolverConfig = merge(
    {
      preserveSymlink: true,
    },
    // @ts-expect-error
    Arena.Workspace?.config?.client?.javascript?.resolve,
    pick(options || {}, "preserveSymlink", "alias", "conditions", "dedupe")
  );

  const resolver = new Resolver(resolverConfig);
  const transpiler = new Transpiler(resolverConfig, env);

  return {
    async load(filePath: string) {
      const filename = resolver.resolve(filePath, "./");
      if (!filename || !Arena.fs.existsSync(filename)) {
        return undefined;
      }

      const ext = path.extname(filename);
      if (!ext) {
        return;
      }
      const contentType = getType(
        // Note(sagar): mime of .ts(x) and .jsx file should be
        // application/javascript, but `mime` lib returns text/jsx.
        // so, manually override
        ext.startsWith(".ts") || ext.startsWith(".js") ? "js" : ext
      )!;

      let content;
      let responseContentType = contentType;
      if ([".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"].includes(ext)) {
        content = await transpiler.transpiledJavascript(filename, ext);
      } else if ([".css"].includes(ext)) {
        content = await transpiler.transformedCss(filename, ext);
        responseContentType = "application/javascript";
      } else {
        if (["application/json"].includes(contentType)) {
          content = await Arena.fs.readToString(filename);
        } else {
          throw new Error("Unsupported file type:" + ext);
        }
      }

      return {
        contentType: responseContentType!,
        code: content,
      };
    },
  };
};

/**
 * This creates the ESM module router which returns `Response` if the
 * file is found. The file path from the route would be treated as relative
 * to the root path of the resolver.
 */
const createModuleRouter = (
  options: ModuleLoaderOptions,
  pathAlias: Record<string, string> = {}
) => {
  const moduleloader = createModuleLoader(options);
  return async ({ event }: any) => {
    let path = event.ctx.path;
    path = pathAlias[path] ?? "./" + path;
    try {
      const res = await moduleloader.load(path);
      if (res) {
        return new Response(res.code, {
          status: 200,
          headers: [["content-type", res.contentType]],
        });
      }
    } catch (e) {}
  };
};

export { createModuleLoader, createModuleRouter };
