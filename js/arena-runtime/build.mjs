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

program.option("--minify").action(async (options, cmd) => {
  await Promise.all([
    build({
      ...options,
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
      entryPoints: {
        crypto: "libs/node/crypto/crypto.ts",
      },
      external: ["buffer"],
      outdir: "dist/node",
    }),

    build({
      ...options,
      minify: true,
      entryPoints: {
        "wasmer-wasi": "./libs/wasi/wasmer.ts",
      },
      format: "iife",
    }),
    build({
      ...options,
      minify: true,
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
      ...options,
      minify: true,
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
        dqs: "./libs/dqs/index.ts",
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
    /**
     * This bundles exports of `@arena/functions/...` so that it can be
     * embedded in DQS server during build time to avoid file access during
     * runtime.
     *
     * `@arena/functions` could be open-sourced later, so, putting the build
     * setup here.
     */
    stdinBuild({
      entryPoints: {
        "router.js": `export * from "@arena/functions/router";`,
        "sql/postgres.js": `export * from "@arena/functions/sql/postgres";`,
      },
      outdir: "dist/functions",
      alias: {
        pg: "@arena/runtime/postgres",
      },
      external: ["@arena/runtime/postgres"],
    }),
    build({
      ...options,
      entryPoints: {
        "app-server": "./libs/apps/server.ts",
      },
      external: [
        "path",
        "crypto",
        "@app/template",
        "@arena/runtime/server",
        "@arena/runtime/sqlite",
        "~/setup/migrations",
      ],
    }),
  ]);
});

program.parse();
