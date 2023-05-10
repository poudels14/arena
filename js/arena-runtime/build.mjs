import { program } from "commander";
import * as esbuild from "esbuild";

const build = async (options) => {
  const { external = [], ...restOptions } = options;
  try {
    await esbuild.build({
      bundle: true,
      outdir: "dist",
      format: "esm",
      external: ["node:*", ...external],
      ...restOptions,
    });
  } catch (e) {
    console.error(e);
    throw e;
  }
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
      entryPoints: {
        "wasmer-wasi": "./libs/wasi/wasmer.ts",
      },
      format: "iife",
    }),
    build({
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
        server: "./libs/server/index.ts",
      },
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
  ]);
});

program.parse();
