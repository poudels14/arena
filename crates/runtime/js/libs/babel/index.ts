import * as babel from "@babel/standalone";
import solidjs from "babel-preset-solid";
import transformCommonJs from "./plugins/transform-commonjs";
import importResolver from "./plugins/import-resolver";

export {
  loadPartialConfig,
  loadPartialConfigAsync,
  transformSync,
  transformAsync,
} from "@babel/core";

const presets = {
  solidjs,
};

const plugins = {
  transformCommonJs,
  importResolver,
};

export { babel, presets, plugins };
