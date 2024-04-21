import type { ConfigEnv, UserConfig } from "vite";
import { defineConfig, mergeConfig } from "vite";
import {
  getBuildConfig,
  external,
  pluginHotRestart,
  copyFilesToOutputDir,
} from "./vite.base.config";
import pkg from "./package.json";

// https://vitejs.dev/config
export default defineConfig((env) => {
  const forgeEnv = env as ConfigEnv<"build">;
  const { forgeConfigSelf } = forgeEnv;
  const config: UserConfig = {
    define: {
      _APP_VERSION: JSON.stringify(pkg.version),
    },
    assetsInclude: ["**/*.node"],
    build: {
      rollupOptions: {
        external,
        input: forgeConfigSelf.entry!,
        output: {
          format: "cjs",
          // It should not be split chunks.
          inlineDynamicImports: true,
          entryFileNames: "[name].js",
          chunkFileNames: "[name].js",
          assetFileNames: "[name].[ext]",
        },
      },
    },
    plugins: [
      pluginHotRestart("reload"),
      copyFilesToOutputDir({
        files: [`../../crates/target/release/portal-${pkg.version}.node`],
        ignoreError: true,
      }),
    ],
  };

  return mergeConfig(getBuildConfig(forgeEnv), config);
});
