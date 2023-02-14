import * as babel from "@babel/standalone";
import solidPreset from "babel-preset-solid";
import transformCommonJs from "./plugins/transform-commonjs";

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
  },
});
