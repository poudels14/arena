import path from "path";
import { presets } from "@arena/runtime/babel";
import { build as rollupBuild, plugins } from "@arena/runtime/rollup";

type BuildConfig = {
  env?: Record<string, any>;
  javascript?: {
    resolve?: {
      alias?: Record<string, string>;
      conditions?: string[];
      dedupe?: string[];
    };
  };
};

/**
 * Build workspace
 */
const build = async (options: {
  outDir: string;
  client?: {
    entry: string;
    config?: BuildConfig;
    minify?: boolean;
  };
  server?: {
    entry: string;
    config?: BuildConfig;
  };
  hydratable?: boolean;
}) => {
  const { outDir, hydratable = false } = options;
  if (options.server) {
    await buildServer(options.server.config, {
      input: options.server.entry,
      outputFile: path.join(outDir, "server/index.js"),
      hydratable,
    });
  }

  if (options.client) {
    const { entry, config, minify } = options.client;
    await buildClient(config, {
      input: entry,
      outDir: path.join(outDir, "static"),
      hydratable,
      minify,
    });
  }
};

const buildServer = async (
  serverConfig: BuildConfig = {},
  options: {
    input: string;
    outputFile: string;
    hydratable: boolean;
  }
) => {
  console.log(`[server]: Starting build...`);
  const start = performance.now();
  const { input, outputFile, hydratable } = options;
  await rollupBuild({
    input,
    output: {
      format: "es",
      inlineDynamicImports: true,
      file: outputFile,
    },
    plugins: [
      plugins.arenaResolver({
        ...(serverConfig.javascript?.resolve || {}),
      }),
      plugins.arenaLoader({}),
      plugins.babel({
        extensions: [".js", ".ts", ".jsx", ".tsx"],
        babelrc: false,
        babelHelpers: "bundled",
        presets: [
          [
            presets.solidjs,
            {
              generate: "ssr",
              hydratable,
            },
          ],
        ],
      }),
      plugins.postcss({
        plugins: [],
      }),
    ],
  });
  console.log(`[server]: Time taken =`, performance.now() - start);
};

const buildClient = async (
  clientConfig: BuildConfig = {},
  options: {
    input: string;
    outDir: string;
    hydratable: boolean;
    minify?: boolean;
  }
) => {
  console.log(`[client]: Starting build...`);
  const start = performance.now();
  const { input, outDir, hydratable } = options;

  const { entries, fromEntries } = Object;
  const envReplace = fromEntries(
    entries(clientConfig.env || {}).flatMap(([k, v]) => {
      return [
        [`Arena.env.${k}`, JSON.stringify(v)],
        [`process.env.${k}`, JSON.stringify(v)],
      ];
    })
  );

  await rollupBuild({
    input,
    output: {
      format: "es",
      dir: outDir,
    },
    plugins: [
      plugins.arenaResolver({
        ...(clientConfig.javascript?.resolve || {}),
      }),
      plugins.arenaLoader({
        replace: envReplace,
      }),
      plugins.babel({
        extensions: [".js", ".ts", ".jsx", ".tsx"],
        babelrc: false,
        babelHelpers: "bundled",
        presets: [
          [
            presets.solidjs,
            {
              generate: "dom",
              hydratable,
            },
          ],
        ],
      }),
      plugins.postcss({
        plugins: [],
      }),
      options.minify && plugins.terser(),
    ],
  });
  console.log(`[client]: Time taken =`, performance.now() - start);
};

export { build };
