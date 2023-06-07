import { defineConfig, install, tw, css, apply } from "@twind/core";
import type { Preset } from "@twind/core";
import presetAutoprefix from "@twind/preset-autoprefix";
import presetTailwind from "@twind/preset-tailwind/base";
import * as tailwindColor from "@twind/preset-tailwind/colors";
import {
  mauve,
  gray,
  slateDark,
  grayDark,
} from "@twind/preset-radix-ui/colors";
import "./style.css";

type Config = {
  presets?: Preset[];
};

const setupTwind = (config: Config = {}) => {
  install(
    defineConfig({
      hash: false,
      presets: [
        presetAutoprefix(),
        presetTailwind({
          colors: {
            brand: mauve,
            brandDark: slateDark,
            accent: gray,
            accentDark: grayDark,
            gray: tailwindColor.gray,
            green: tailwindColor.green,
            red: tailwindColor.red,
            cyan: tailwindColor.cyan,
            blue: tailwindColor.blue,
          },
        }),
        ...(config.presets || []),
      ],
      rules: [
        /**
         * This rule allows styling nested elements in a widget.
         * To set style for a nested element, a class to the widget in the
         * following supported format should be added:
         *
         * art-[{selector}]({utility-class},{separated-by-comma})
         *
         * Example widget:
         *
         * const Template = () => <div class="table">
         *  <div class="thead">{...}</div>
         *  <div class="tbody">{...}</div>
         * <div>;
         *
         * To configure top level component, add class:
         *  - art-(text-red-800)
         *
         * To configure child element, add class:
         *  - art-[>.tbody](bg-red-300)
         *
         *  - Note: when selecting child element, the selector should start with `>`
         *
         * There's no limit to nesting.
         */
        [
          "art-",
          ({ input }) => {
            /**
             * Note(sagar): the split returns ['', {selector}, {util-class}]
             * Since selector is optional, reverse the array such that it's
             * easier to patten match
             */
            const [style, selector, _] = input
              .substring(4)
              .split(/\[|\]-/)
              .reverse();

            const css = {
              "@apply": style,
            };
            if (selector) {
              const path: any[] = selector.split(/>/);
              return path.reduceRight((agg, p, idx) => {
                if (idx === 0) {
                  return { ":where(&)": agg };
                } else {
                  return { [`&>${p}`]: agg };
                }
              }, css);
            }
            return css;
          },
        ],
      ],
    })
  );
};

const twind = {
  tw,
  css,
  apply,
};

export { twind };
export { setupTwind };
