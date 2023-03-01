import { createHandler } from "@arena/core/server";
import * as mime from "mime";
import * as path from "path";

const { Transpiler } = Arena.BuildTools;
const transpiler = new Transpiler({
  resolve_import: true,
  resolver: {
    preserve_symlink: true,
    conditions: ["browser", "development"],
  },
  replace: {
    "Arena.env.MODE": JSON.stringify(Arena.env.MODE),
    // Note(sagar): SSR should always be false since this
    // transpiler is used for browser code
    "Arena.env.SSR": JSON.stringify(false),
    "Arena.env.ARENA_SSR": JSON.stringify(Arena.env.ARENA_SSR),
  },
});

export default createHandler(async (event) => {
  const filePath = event.ctx.path;
  const ext = path.extname(filePath);
  if (!ext) {
    return;
  }
  const contentType = mime.getType(ext.startsWith(".ts") ? "js" : ext);

  const filename = "./" + filePath;
  if (!Arena.fs.existsSync(filename)) {
    return;
  }

  try {
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

    return new Response(transpiledCode, {
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
