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
          tty: "libs/node/tty.ts",
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
          "process.js": createNodejsModule("process"),
          "url.js": createNodejsModule("url"),
          "os.js": createNodejsModule("os"),
          "stream.js": createNodejsModule("stream"),
          "util.js": createNodejsModule("util"),
          "stream.js": createNodejsModule("stream"),
          "buffer.js": createNodejsModule("buffer"),
        },
        outdir: "dist/node",
        external: ["stream"],
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
          postgres: "./libs/postgres/index.ts",
        },
      }),
    ]);
  });

program.parse();
