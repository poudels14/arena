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
        os: "node:os",
        "@babel/core": "@arena/babel",
      },
      external: ["fs", "tty", "stream", "@arena/babel"],
    }),
  ]);
});

program.parse();
