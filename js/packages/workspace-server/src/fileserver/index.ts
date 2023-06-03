import { createHandler } from "@arena/core/server";
import { Transpiler } from "@arena/runtime/transpiler";
import { babel, presets, plugins } from "@arena/runtime/babel";
import * as mime from "mime";
import * as path from "path";
import { isMatch } from "matcher";

const createFileServer = () => {
  // @ts-ignore
  const { resolve } = Arena.Workspace?.config?.client?.javascript || {};
  const clientEnv = Object.assign(
    {
      // Note(sagar): since this is client env, always set SSR = true
      SSR: false,
    },
    Arena.env,
    // @ts-ignore
    Arena.Workspace?.config?.client?.env
  );

  const transpiler = new Transpiler({
    resolve_import: true,
    resolver: {
      preserve_symlink: resolve?.preserve_symlink || true,
      conditions: resolve?.conditions || ["browser", "development"],
      dedupe: resolve?.dedupe,
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

  return createHandler(async (event) => {
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
      let responseContentType = contentType;
      if ([".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"].includes(ext)) {
        content = await getTranspiledJavascript(transpiler, filename, ext);
      } else if ([".css"].includes(ext)) {
        content = await getTransformedCss(transpiler, filename, ext);
        responseContentType = "application/javascript";
      } else {
        if (["application/json"].includes(contentType)) {
          content = await Arena.fs.readToString(filename);
        } else {
          throw new Error("Unsupported file type:" + ext);
        }
      }

      return new Response(content, {
        headers: {
          "content-type": responseContentType!,
        },
        status: 200,
      });
    } catch (e) {
      console.error(e);
      throw e;
    }
  });
};

const TRANSPILED_CODE_BY_FILENAME = new Map();
const TRANSPILED_FILED_LAST_MODIFIED_TIME = new Map();
const getTranspiledJavascript = async (
  transpiler: Transpiler,
  filePath: string,
  ext: string
) => {
  if (TRANSPILED_CODE_BY_FILENAME.has(filePath)) {
    // only return cached transpiled code if the file hasn't been modified
    // after it was transpiled
    const { mtimeMs } = Arena.fs.lstatSync(filePath);
    if (mtimeMs == TRANSPILED_FILED_LAST_MODIFIED_TIME.get(filePath)) {
      return TRANSPILED_CODE_BY_FILENAME.get(filePath);
    }
  }

  const { code } = await transpiler.transpileFileAsync("./" + filePath);
  let transpiledCode = code;

  // Note(sagar): further transpile JSX using babel plugins
  if ([".tsx", ".jsx"].includes(ext)) {
    const { code: solidjsCode } = babel.transform(code, {
      presets: [[presets.solidjs, { generate: "dom", hydratable: true }]],
      plugins: [
        [plugins.transformCommonJs, { exportsOnly: true }],
        plugins.importResolver,
      ],
    });
    transpiledCode = solidjsCode;
  }

  if (isMatch(filePath, "*/node_modules/*")) {
    TRANSPILED_CODE_BY_FILENAME.set(filePath, transpiledCode);
    const { mtimeMs } = Arena.fs.lstatSync(filePath);
    TRANSPILED_FILED_LAST_MODIFIED_TIME.set(filePath, mtimeMs);
  }
  return transpiledCode;
};

const getTransformedCss = async (
  transpiler: Transpiler,
  filePath: string,
  ext: string
) => {
  const css = await Arena.fs.readToString("./" + filePath);
  const { code } = await transpiler.transpileSync(
    `import styleInject from "style-inject";
const css = \`${css}\`;
styleInject(css);
export default css;
`,
    filePath
  );
  // TODO(sagar): transform using postcss
  return code;
};

export { createFileServer };
