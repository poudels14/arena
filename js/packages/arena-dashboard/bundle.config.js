import path from "path";
import {
  client as buildClient,
  server as buildServer,
} from "@arena/runtime/bundler";
import { presets } from "@arena/runtime/babel";
import { plugins } from "@arena/runtime/rollup";
import { merge } from "lodash-es";
import pkg from "./package";
import { BUILTIN_APPS } from "./src/BUILTIN_APPS";
const { babel, postcss, terser } = plugins;

/**
 * This is super hacky way to build a separate bundle for each builtin
 * app template. Remove this once a better workflow is in place.
 */
const BUILTIN_APP_CLIENT_ENTRIES = BUILTIN_APPS.reduce((agg, app) => {
  agg[
    `templates/apps/${app.id}/${app.version}`
  ] = `./src/${app.id}/src/root.tsx`;
  return agg;
}, {});

const BUILTIN_APP_SERVER_ENTRIES = BUILTIN_APPS.reduce((agg, app) => {
  agg[`templates/apps/${app.id}/${app.version}`] = `./src/${app.id}/server.ts`;
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
        ...BUILTIN_APP_CLIENT_ENTRIES,
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
      env: {
        // Note(sagar): this is loaded from package.json/"arena" config
        ...env,
        ...process.env,
      },
      javascript: merge(javascript, {
        resolve: {
          conditions: ["browser", "solid"],
        },
      }),
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
    await buildServerBundle(
      {
        index: entry,
      },
      javascript
    );
    await Promise.all(
      Object.entries(BUILTIN_APP_SERVER_ENTRIES).map(async ([k, v]) => {
        await buildServerBundle(
          {
            [k]: v,
          },
          javascript
        );
      })
    );
  }
}

const buildServerBundle = async (input, javascript) => {
  await buildServer({
    input,
    output: {
      format: "es",
      entryFileNames: "[name].js",
      inlineDynamicImports: true,
      dir: path.join(outDir, "server/"),
    },
    javascript,
    replace: {
      // Note(sagar): this is to treeshake dev related code in non-dev mode
      "process.env.MODE": JSON.stringify(process.env.MODE),
    },
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
};
