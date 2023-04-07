import * as babel from "@babel/standalone";
import solidPreset from "babel-preset-solid";
import transformCommonJsPlugin from "./plugins/transform-commonjs";
import importResolverPlugin from "./plugins/import-resolver";

if (!globalThis.Arena.BuildTools) {
  throw new Error("Arena.BuildTools is undefined");
}

Object.assign(globalThis.Arena.BuildTools, {
  babel,
  babelPresets: {
    solid: solidPreset,
  },
  babelPlugins: {
    transformCommonJs: transformCommonJsPlugin,
    importResolver: importResolverPlugin,
  },
});

export {
  loadPartialConfig,
  loadPartialConfigAsync,
  transformSync,
  transformAsync,
} from "@babel/core";
export { babel, solidPreset, transformCommonJsPlugin, importResolverPlugin };
