import path from "path";
import {
  client as buildClient,
  server as buildServer,
} from "@arena/workspace-server/builder";
import { presets } from "@arena/runtime/babel";
import { plugins } from "@arena/runtime/rollup";
const { babel, postcss, terser } = plugins;

const hydratable = false;
const outDir = "./build";
export default async function (options) {
  if (options.client) {
    await buildClient({
      input: options.client.entry,
      output: {
        format: "es",
        dir: path.join(outDir, "static"),
      },
      env: options.client.env,
      javascript: options.client.javascript,
      plugins: [
        babel({
          extensions: [".js", ".ts", ".jsx", ".tsx"],
          babelrc: false,
          babelHelpers: "bundled",
          presets: [
            [
              presets.solidjs,
              {
                generate: "dom",
                hydratable,
              },
            ],
          ],
        }),
        postcss({
          plugins: [],
        }),
        terser(),
      ],
    });
  }

  if (options.server) {
    await buildServer({
      input: options.server.entry,
      output: {
        format: "es",
        inlineDynamicImports: true,
        file: path.join(outDir, "server/index.js"),
      },
      javascript: options.server.javascript,
      plugins: [
        babel({
          extensions: [".js", ".ts", ".jsx", ".tsx"],
          babelrc: false,
          babelHelpers: "bundled",
          presets: [
            [
              presets.solidjs,
              {
                generate: "ssr",
                hydratable,
              },
            ],
          ],
        }),
        postcss({
          plugins: [],
        }),
      ],
    });
  }
}
