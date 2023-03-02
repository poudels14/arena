// credit: solid-start
import { JSX } from "solid-js";
import { hydrate, render } from "solid-js/web";
import { defineConfig, install } from "@twind/core";
import presetAutoprefix from "@twind/preset-autoprefix";
import presetTailwind from "@twind/preset-tailwind";
import presetRadixUi from "@twind/preset-radix-ui";

if (Arena.env.MODE === "development") {
  install(
    defineConfig({
      presets: [presetAutoprefix(), presetTailwind(), presetRadixUi()],
    })
  );
}

const mount = (code?: () => JSX.Element, element?: Document) => {
  if (Arena.env.ARENA_SSR) {
    code && element && hydrate(code, element);
  } else {
    code &&
      element &&
      render(code, element === document ? element.body : element);
  }
};

export { mount };
