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
    options.minify = !options.dev;
    delete options.dev;

    await Promise.all([
      build({
        ...options,
        minify: options.minify,
        entryPoints: {
          "workspace/migrate": "./migrate.ts",
        },
        external: ["path", "process", "crypto", "@arena/runtime/postgres"],
      }),
    ]);
  });

program.parse();
