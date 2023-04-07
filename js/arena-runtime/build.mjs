import { program } from "commander";
import * as esbuild from "esbuild";
import glob from "glob";

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
    glob(["libs/node/*"]).then((entryPoints) =>
      build({
        ...options,
        entryPoints,
        outdir: "dist/node",
      })
    ),
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
        "@babel/core": "@arena/babel",
      },
      external: ["tty", "crypto", "module", "stream", "@arena/babel"],
    }),
    build({
      ...options,
      entryPoints: {
        buildtools: "./libs/buildtools.ts",
      },
      external: ["@arena/babel"],
    }),
  ]);
});

program.parse();
