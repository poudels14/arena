export const rollup: any;
export const plugins: {
  terser: () => any;
  arenaResolver: (options: ResolverConfig) => any;
  arenaLoader: (options: { replace?: TranspilerConfig["replace"] }) => any;
  babel: (options: any) => any;
  postcss: (options: any) => any;
};
export const build: (options: any) => Promise<void>;
