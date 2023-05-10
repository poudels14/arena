// Note(sagar): build-tools should be enabled for this
import { presets } from "@arena/runtime/babel";
import { build, plugins } from "@arena/runtime/rollup";

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
          presets.solidjs,
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
