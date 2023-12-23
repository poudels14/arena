import { program } from "commander";
import path from "path";
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

/**
 * Bundle using given entry code;
 * options.entryPoints is map of `outfile => code` instead of
 * `outfile => entryFile`
 *
 * @param {BuildOptions} options
 */
const stdinBuild = async (options) => {
  await Promise.all(
    Object.entries(options.entryPoints).map(async ([outfile, code]) => {
      await build({
        ...options,
        outfile: path.join(options.outdir ?? "", outfile),
        outdir: undefined,
        entryPoints: undefined,
        stdin: {
          resolveDir: process.cwd(),
          contents: code,
        },
      });
    })
  );
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
          "dqs/utils": "./libs/dqs/utils/index.ts",
          "dqs/widget-server": "./libs/dqs/widget/server.ts",
          "dqs/postgres": "./libs/dqs/postgres/index.ts",
          "dqs/app-server": "./libs/dqs/app/server.ts",
          "dqs/plugin-workflow": "./libs/dqs/plugin/workflow/index.ts",
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
          "@arena/dqs/utils",
          "@arena/runtime/server",
          "@arena/runtime/sqlite",
          "@arena/runtime/postgres",
          "@arena/cloud",
          "~/setup/migrations",
        ],
      }),
      stdinBuild({
        minify: true,
        ...options,
        entryPoints: {
          // TODO(sagar): "assert" is being bundled in several files, fix it
          "vectordb.js": getCloudExportCode("vectordb"),
          "jwt.js": getCloudExportCode("jwt"),
          "pubsub.js": getCloudExportCode("pubsub"),
          "query.js": getCloudExportCode("query"),
          "llm.js": getCloudExportCode("llm"),
          "html.js": getCloudExportCode("html"),
          "pdf.js": getCloudExportCode("pdf"),
        },
        outdir: "dist/cloud",
      }),
    ]);
  });

const getCloudExportCode = (pkg) => {
  return `export * from "@arena/cloud/${pkg}";`;
};

program.parse();
