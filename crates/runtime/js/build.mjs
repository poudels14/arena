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
          assert: "libs/node/assert.ts",
          events: "libs/node/events.ts",
          fs: "libs/node/fs.ts",
          "fs/promises": "libs/node/fs_promises.ts",
          url: "libs/node/url.ts",
          path: "libs/node/path.ts",
          perf_hooks: "libs/node/perf_hooks.ts",
          process: "libs/node/process.ts",
          tty: "libs/node/tty.ts",
          util: "libs/node/util.ts",
          buffer: "libs/node/buffer.ts",
        },
        outdir: "dist/node",
      }),
      build({
        ...options,
        minify: false,
        entryPoints: {
          crypto: "libs/node/crypto/index.ts",
        },
        external: ["buffer"],
        outdir: "dist/node",
      }),

      build({
        minify: true,
        ...options,
        entryPoints: {
          "wasmer-wasi": "./libs/wasi/wasmer.ts",
        },
        format: "iife",
      }),
      build({
        minify: true,
        ...options,
        entryPoints: {
          babel: "./libs/babel/index.ts",
        },
        alias: {
          // Note(sagar): need to alias these to a file to avoid dynamic import
          fs: "./libs/alias/fs.ts",
          path: "./libs/alias/path.ts",
        },
        external: ["@arena/runtime/resolver"],
      }),
      build({
        minify: true,
        ...options,
        entryPoints: {
          rollup: "./libs/rollup/index.ts",
        },
        alias: {
          // Note(sagar): Even though arena runtime doesn't have 'os' module,
          // need to alias this here so that build is successful. I think the
          // bundle doesn't need os module because watch and any other code that
          // uses 'os' module isn't being included in the bundle

          os: "./libs/alias/os.ts",
          fs: "./libs/alias/fs.ts",
          resolve: "./libs/alias/resolve.ts",
          module: "./libs/alias/module.ts",
          "postcss-load-config": "./libs/alias/postcss-load-config.ts",
          "@babel/core": "@arena/runtime/babel",
        },
        external: [
          "tty",
          "crypto",
          "stream",
          "@arena/runtime/babel",
          "@arena/runtime/resolver",
          "@arena/runtime/transpiler",
        ],
      }),
      build({
        ...options,
        entryPoints: {
          resolver: "./libs/resolver.ts",
        },
      }),
      build({
        ...options,
        entryPoints: {
          transpiler: "./libs/transpiler.ts",
        },
      }),
      build({
        ...options,
        entryPoints: {
          server: "./libs/server/index.ts",
        },
      }),
      build({
        ...options,
        entryPoints: {
          bundler: "./libs/bundler/index.ts",
        },
        external: ["@arena/runtime/rollup"],
      }),
      build({
        minify: true,
        ...options,
        entryPoints: {
          filerouter: "./libs/filerouter/index.ts",
        },
        external: [
          "@arena/runtime/resolver",
          "@arena/runtime/transpiler",
          "@arena/runtime/babel",
        ],
      }),
    ]);
  });

program.parse();
