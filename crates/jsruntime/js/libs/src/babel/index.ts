import * as babel from "@babel/standalone";
import solidPreset from "babel-preset-solid";
import transformCommonJs from "./plugins/transform-commonjs";

if (!globalThis.Arena) {
  globalThis.Arena = {};
}

globalThis.Arena.babel = babel;
globalThis.Arena.babelPresets = {
  solid: solidPreset,
};
globalThis.Arena.babelPlugins = {
  transformCommonJs,
};
