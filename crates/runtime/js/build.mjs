import { program } from "commander";
import path from "path";
import * as esbuild from "esbuild";
import { camelCase } from "lodash-es";

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

const createNodejsModule = (moduleName, entryPath) => {
  const name = camelCase(moduleName);
  const path = `./libs/node/${entryPath || moduleName}.js`;
  return `
  import * as ${name} from '${path}';


  let def = ${name};
  if (${name}.default) {
    def = Object.assign(${name}.default, ${name});
  }
  Arena.__nodeInternal = {
    ...(Arena.__nodeInternal || {}),
    "${moduleName}": def,
  };
  export * from '${path}';
  export default def;
  `;
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
        entryPoints: {
          perf_hooks: "libs/node/perf_hooks.ts",
          process: "libs/node/process.ts",
          tty: "libs/node/tty.ts",
          buffer: "libs/node/buffer.ts",
        },
        outdir: "dist/node",
      }),
      stdinBuild({
        ...options,
        entryPoints: {
          "path.js": createNodejsModule("path"),
          "assert.js": createNodejsModule("assert"),
          "constants.js": createNodejsModule("constants"),
          "fs.js": createNodejsModule("fs", "fs/index"),
          "fs/promises.js": createNodejsModule("fs/promises"),
          "events.js": createNodejsModule("events"),
          "url.js": createNodejsModule("url"),
          "os.js": createNodejsModule("os"),
          "stream.js": createNodejsModule("stream"),
          "util.js": createNodejsModule("util"),
          "stream.js": createNodejsModule("stream"),
        },
        outdir: "dist/node",
      }),
      build({
        ...options,
        entryPoints: {
          crypto: "libs/node/crypto/index.ts",
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
    ]);
  });

program.parse();
