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
          "dqs/widget-server": "./dqs/widget/server.ts",
          "dqs/postgres": "./dqs/postgres/index.ts",
          "dqs/app-server": "./dqs/app/server.ts",
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
          jwt: "./cloud/jwt.ts",
          s3: "./cloud/s3.ts",
          pubsub: "./cloud/pubsub/index.ts",
          query: "./cloud/query/index.ts",
          llm: "./cloud/llm/index.ts",
          html: "./cloud/html/index.ts",
          pdf: "./cloud/pdf/index.ts",
          pyodide: "./cloud/pyodide/index.ts",
          "pyodide/pyodide.asm": "./cloud/pyodide/pyodide.asm.js",
        },
        outdir: "dist/cloud",
        external: ["crypto", "fs", "fs/promises", "path", "tty", "url"],
      }),
    ]);
  });

program.parse();
