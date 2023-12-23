import { getType } from "mime";
import path from "node:path";
import pick from "lodash-es/pick";
import merge from "lodash-es/merge";
import { Resolver } from "@arena/runtime/resolver";
import { Transpiler } from "./transpiler";

type FileLoaderOptions = {
  env?: Record<string, string>;
  resolve?: Arena.ResolverConfig;
};

/**
 * This creates an ESM module loader that returns transpiled JS/TS
 * files given the file path
 */
const createFileLoader = (options: FileLoaderOptions) => {
  const env = merge(process.env, options.env);
  const resolverConfig = merge(
    {
      preserveSymlink: true,
    },
    pick(
      options.resolve || {},
      "preserveSymlink",
      "alias",
      "conditions",
      "dedupe"
    )
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
        responseContentType = "application/javascript";
        content = await transpiler.transpiledJavascript(filename, ext);
      } else if ([".css"].includes(ext)) {
        content = await transpiler.transformedCss(filename, ext);
        responseContentType = "application/javascript";
      } else {
        if (["application/json"].includes(contentType)) {
          content = await Arena.fs.readToString(filename);
        } else {
          content = await Arena.fs.readFile(filename);
        }
      }

      return {
        contentType: responseContentType!,
        content,
      };
    },
  };
};

/**
 * This creates the ESM module router which returns `Response` if the
 * file is found. The file path from the route would be treated as relative
 * to the root path of the resolver.
 */
const createFileRouter = (options: FileLoaderOptions) => {
  const fileloader = createFileLoader(options);
  return async (req: Request) => {
    const url = new URL(req.url);
    let path = url.pathname;
    // Note(sagar): since `path` starts with `/` but we want all the paths to
    // be relative to the project root, prefix it with `.` if path isn't in
    // pathAlias
    path = "." + path;
    try {
      const res = await fileloader.load(path);
      if (res) {
        return new Response(res.content, {
          status: 200,
          headers: [["content-type", res.contentType]],
        });
      }
    } catch (e) {}
    /**
     * Note(sagar): return 404 for favicon if not found. This is to prevent
     * next router from being called if file router doesn't return anything.
     */
    if (url.pathname == "/favicon.ico") {
      return new Response("Not found", {
        status: 404,
      });
    }
  };
};

export { createFileRouter };
