import { program } from "commander";
import * as esbuild from "esbuild";

/**
 * @typedef {{
 *  entryPoints: Record<string, string>;
 *  external?: string[];
 *  outdir?: string;
 *  minify?: boolean;
 *  format?: "esm" | "iife";
 * }} BuildOptions
 */

/**
 * @param {BuildOptions} options
 */
const build = async (options) => {
  const { external = [], ...restOptions } = options;
  try {
    await esbuild.build({
      bundle: true,
      outdir: restOptions.outfile ? undefined : "dist",
      format: "esm",
      external: ["node:*", ...external],
      ...restOptions,
    });
  } catch (e) {
    console.error(e);
    throw e;
  }
};

program
  .option("--minify")
  .option("--dev")
  .action(async (options, cmd) => {
    if (options.dev) {
      options.minify = false;
      delete options.dev;
    }

    await Promise.all([
      build({
        ...options,
        minify: false,
        entryPoints: {
          "dqs/utils": "./dqs/utils/index.ts",
          "dqs/widget-server": "./dqs/widget/server.ts",
          "dqs/postgres": "./dqs/postgres/index.ts",
          "dqs/app-server": "./dqs/app/server.ts",
          "dqs/plugin-workflow": "./dqs/plugin/workflow/index.ts",
          "dqs/plugin/workflow/lib": "./dqs/plugin/workflow/lib.ts",
        },
        alias: {
          // TODO(sagar): "assert" and "utils" are being bundled in several files
          // but couldn't mark it was external because it would use "require" when
          // marking as external. Using alias didn't work either. Figure out sth
          pg: "@arena/runtime/postgres",
        },
        external: [
          "path",
          "crypto",
          "@dqs/template/app",
          "@dqs/template/plugin",
          "@arena/dqs/plugin/workflow/lib",
          "@arena/dqs/utils",
          "@arena/runtime/server",
          "@arena/runtime/sqlite",
          "@arena/runtime/postgres",
          "@arena/cloud",
          "~/setup/migrations",
        ],
      }),
      build({
        minify: true,
        ...options,
        entryPoints: {
          // TODO(sagar): "assert" is being bundled in several files, fix it
          "vectordb.js": "./cloud/vectordb/index.ts",
          "jwt.js": "./cloud/jwt.ts",
          "pubsub.js": "./cloud/pubsub/index.ts",
          "query.js": "./cloud/query/index.ts",
          "llm.js": "./cloud/llm/index.ts",
          "html.js": "./cloud/html/index.ts",
          "pdf.js": "./cloud/pdf/index.ts",
        },
        outdir: "dist/cloud",
      }),
    ]);
  });

program.parse();
