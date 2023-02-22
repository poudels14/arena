import { createHandler } from "@arena/core/server";
import * as path from "path";

const { BuildTools } = Arena;
export default createHandler(async (event) => {
  const ext = path.extname(event.ctx.path);
  if (!ext) {
    return;
  }

  try {
    console.log(Arena.env);
    const { code } = await BuildTools.transformFileAsync(
      "./" + Arena.env.ARENA_ENTRY_CLIENT,
      {}
    );
    const { babel, babelPlugins, babelPresets } = Arena.BuildTools;
    const { code: transpiledCode } = babel.transform(code, {
      presets: [
        [babelPresets.solid, { generate: "dom", hydratable: true }],
      ],
      plugins: [[babelPlugins.transformCommonJs, { exportsOnly: true }]],
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
