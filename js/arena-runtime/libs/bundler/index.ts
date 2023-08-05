import { build as rollup, plugins } from "@arena/runtime/rollup";

type BuildConfig = {
  env?: Record<string, any>;
  javascript?: {
    resolve?: {
      alias?: Record<string, string>;
      conditions?: string[];
      dedupe?: string[];
      external?: string[];
    };
  };
};

/**
 *
 * @param options rollup options
 */
const buildServer = async (options: {
  input: string;
  output: any;
  javascript?: BuildConfig["javascript"];
  replace: Record<string, any>;
  // rollup plugins
  plugins?: any[];
}) => {
  options = Object.assign(
    {
      javascript: {},
      plugins: [],
    },
    options
  ) as Required<typeof options>;
  console.log(`[server]: Starting build...`);
  const start = performance.now();
  await rollup({
    input: options.input,
    output: options.output,
    plugins: [
      plugins.arenaResolver({
        ...(options.javascript?.resolve || {}),
      }),
      plugins.arenaLoader({
        replace: options.replace,
      }),
      ...options.plugins!,
    ],
  });
  console.log(`[server]: Time taken =`, performance.now() - start);
};

const buildClient = async (options: {
  input: string;
  output: any;
  env?: BuildConfig["env"];
  javascript?: BuildConfig["javascript"];
  // rollup plugins
  plugins?: any[];
}) => {
  options = Object.assign(
    {
      env: {},
      javascript: {},
      plugins: [],
    },
    options
  ) as Required<typeof options>;

  console.log(`[client]: Starting build...`);
  const start = performance.now();

  const { entries, fromEntries } = Object;
  const envReplace = fromEntries(
    entries(options.env!).map(([k, v]) => {
      return [`process.env.${k}`, JSON.stringify(v)];
    })
  );

  await rollup({
    input: options.input,
    output: options.output,
    plugins: [
      plugins.arenaResolver({
        ...(options.javascript?.resolve || {}),
      }),
      plugins.arenaLoader({
        replace: envReplace,
      }),
      ...options.plugins!,
    ],
  });
  console.log(`[client]: Time taken =`, performance.now() - start);
};

export { buildClient as client, buildServer as server };
