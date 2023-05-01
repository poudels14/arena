// Note(sagar): build-tools should be enabled for this
import { solidPreset } from "@arena/babel";
import { build, plugins } from "@arena/rollup";

let start = performance.now();

build({
  input: "./solid-js.jsx",
  output: {
    format: "es",
    dir: "build",
  },
  plugins: [
    plugins.babel({
      extensions: [".jsx", ".tsx"],
      babelrc: false,
      babelHelpers: "bundled",
      presets: [
        [
          solidPreset,
          {
            generate: "ssr",
            hydratable: false,
          },
        ],
      ],
    }),
  ],
}).then(() => {
  console.log("Time taken [solid-jsx] =", performance.now() - start);
  start = performance.now();
});
