import * as babel from "@babel/standalone";
import solidPreset from "babel-preset-solid";
import transformCommonJs from "./plugins/transform-commonjs";
import importResolver from "./plugins/import-resolver";

if (!globalThis.Arena.BuildTools) {
  throw new Error("Arena.BuildTools is undefined");
}

Object.assign(globalThis.Arena.BuildTools, {
  babel,
  babelPresets: {
    solid: solidPreset,
  },
  babelPlugins: {
    transformCommonJs,
    importResolver,
  },
});

export {
  loadPartialConfig,
  loadPartialConfigAsync,
  transformSync,
  transformAsync,
} from "@babel/core";
export { solidPreset };
