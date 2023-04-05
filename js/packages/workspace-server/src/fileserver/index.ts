import { createHandler } from "@arena/core/server";
import * as mime from "mime";
import * as path from "path";

// @ts-ignore
const resolve = Arena.Workspace.config?.client?.javascript || {};
const clientEnv = Object.assign(
  {},
  Arena.env,
  // @ts-ignore
  Arena.Workspace.config?.client?.env
);
const { Transpiler } = Arena.BuildTools;
const transpiler = new Transpiler({
  resolve_import: true,
  resolver: {
    preserve_symlink: resolve?.preserve_symlink || true,
    conditions: resolve?.conditions || ["browser", "development"],
    dedupe: resolve?.dedupe || [
      "solid-js",
      "solid-js/web",
      "@solidjs/meta",
      "@solidjs/router",
      "@arena/solid-store",
    ],
  },
  replace: Object.fromEntries(
    Object.entries(clientEnv).flatMap(([k, v]) => {
      return [
        [`Arena.env.${k}`, JSON.stringify(v)],
        [`process.env.${k}`, JSON.stringify(v)],
      ];
    })
  ),
});

const getTranspiledJavascript = async (filePath: string, ext: string) => {
  const { code } = await transpiler.transpileFileAsync("./" + filePath);
  let transpiledCode = code;

  // Note(sagar): further transpile JSX using babel plugins
  if ([".tsx", ".jsx"].includes(ext)) {
    const { babel, babelPlugins, babelPresets } = Arena.BuildTools;
    const { code: solidjsCode } = babel.transform(code, {
      presets: [[babelPresets.solid, { generate: "dom", hydratable: true }]],
      plugins: [
        [babelPlugins.transformCommonJs, { exportsOnly: true }],
        babelPlugins.importResolver,
      ],
    });
    transpiledCode = solidjsCode;
  }
  return transpiledCode;
};

export default createHandler(async (event) => {
  const filePath = event.ctx.path;
  const ext = path.extname(filePath);
  if (!ext) {
    return;
  }
  const contentType = mime.getType(
    // Note(sagar): mime of .ts(x) and .jsx file should be
    // application/javascript, but `mime` lib returns text/jsx.
    // so, manually override
    ext.startsWith(".ts") || ext.startsWith(".js") ? "js" : ext
  )!;

  const filename = "./" + filePath;
  if (!Arena.fs.existsSync(filename)) {
    return;
  }

  try {
    let content;
    if ([".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"].includes(ext)) {
      content = await getTranspiledJavascript(filename, ext);
    } else {
      if (["application/json"].includes(contentType)) {
        content = await Arena.fs.readToString(filename);
      } else {
        throw new Error("Unsupported file type:" + ext);
      }
    }

    return new Response(content, {
      headers: {
        "content-type": contentType!,
      },
      status: 200,
    });
  } catch (e) {
    console.error(e);
    throw e;
  }
});
