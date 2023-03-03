// credit: solid-start
import { JSX } from "solid-js";
import { hydrate, render } from "solid-js/web";
import { defineConfig, install } from "@twind/core";
import presetAutoprefix from "@twind/preset-autoprefix";
import presetTailwind from "@twind/preset-tailwind/base";
import {
  slate,
  slateDark,
  gray,
  grayDark,
} from "@twind/preset-radix-ui/colors";

if (Arena.env.MODE === "development") {
  install(
    defineConfig({
      presets: [
        presetAutoprefix(),
        presetTailwind({
          colors: {
            brand: gray,
            brandDark: slateDark,
            accent: gray,
            accentDark: grayDark,
            neutral: slate,
            neutralDark: slateDark,
            gray: gray,
            grayDark: grayDark,
          },
        }),
      ],
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
