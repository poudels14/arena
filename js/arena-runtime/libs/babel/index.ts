import * as babel from "@babel/standalone";
import solidPreset from "babel-preset-solid";
import transformCommonJsPlugin from "./plugins/transform-commonjs";
import importResolverPlugin from "./plugins/import-resolver";

export {
  loadPartialConfig,
  loadPartialConfigAsync,
  transformSync,
  transformAsync,
} from "@babel/core";
export { babel, solidPreset, transformCommonJsPlugin, importResolverPlugin };
