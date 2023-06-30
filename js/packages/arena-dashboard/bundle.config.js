import path from "path";
import {
  client as buildClient,
  server as buildServer,
} from "@arena/workspace-server/builder";
import { presets } from "@arena/runtime/babel";
import { plugins } from "@arena/runtime/rollup";
import pkg from "./package";
import { BUILTIN_APPS } from "./src/@arena";
const { babel, postcss, terser } = plugins;

/**
 * This is super hacky way to build a separate bundle for each builtin
 * app template. Remove this once a better workflow is in place.
 */
const BUILTIN_APP_ENTRIES = BUILTIN_APPS.reduce((agg, app) => {
  agg[`apps/${app.id}-${app.version}`] = `~/@arena/apps/${app.id.substring(
    "@arena/".length
  )}`;
  return agg;
}, {});

const hydratable = false;
const outDir = "./build";
export default async function (options) {
  if (options.client) {
    const { entry, env, javascript } = options.client;
    await buildClient({
      input: {
        [`arena-${pkg.version}`]: entry,
        ...BUILTIN_APP_ENTRIES,
      },
      output: {
        format: "es",
        dir: path.join(outDir, "static"),
        manualChunks(id) {
          if (
            [
              "node_modules/solid-js",
              "node_modules/@arena/core",
              "node_modules/@arena/uikit",
            ].find((s) => id.includes(s))
          ) {
            return "core";
          }
        },
      },
      env,
      javascript: javascript,
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
    const { entry, javascript } = options.server;
    await buildServer({
      input: entry,
      output: {
        format: "es",
        inlineDynamicImports: true,
        file: path.join(outDir, "server/index.js"),
      },
      javascript,
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

const buildClientBundle = async (options) => {};
