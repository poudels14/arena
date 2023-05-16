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
      entryPoints: {
        router: "./src/router.ts",
      },
      outdir: "dist/",
    }),
    // @arena/core/dqs/... modules
    build({
      ...options,
      entryPoints: {
        postgres: "src/core/postgres.ts",
      },
      outdir: "dist/dqs",
      alias: {
        pg: "@arena/runtime/postgres",
      },
      external: ["@arena/runtime/postgres"],
    }),
  ]);
});

program.parse();
