import { createHandler } from "@arena/core/server";
import * as path from "path";

const { Transpiler } = Arena.BuildTools;
const transpiler = new Transpiler({
  resolve_import: true,
  resolver: {
    conditions: ["browser", "development"],
  },
  replace: {
    "Arena.env.MODE": JSON.stringify("development"),
    "Arena.env.SSR": JSON.stringify(false),
    "Arena.env.ARENA_SSR": JSON.stringify(true),
  },
});

export default createHandler(async (event) => {
  const filePath = event.ctx.path;
  const ext = path.extname(filePath);
  if (!ext) {
    return;
  }

  try {
    const { code } = await transpiler.transpileFileAsync("./" + filePath);

    const { babel, babelPlugins, babelPresets } = Arena.BuildTools;
    const { code: transpiledCode } = babel.transform(code, {
      presets: [[babelPresets.solid, { generate: "dom", hydratable: true }]],
      plugins: [
        [babelPlugins.transformCommonJs, { exportsOnly: true }],
        babelPlugins.importResolver,
      ],
    });

    return new Response(transpiledCode, {
      headers: {
        "content-type": "application/javascript",
      },
      status: 200,
    });
  } catch (e) {
    console.error(e);
  }
});
