import { AppContextProvider } from "@arena/sdk/app";
import { mount, ClientRoot } from "@arena/core/solid/client";
import { setupTwind } from "@arena/uikit/twind";
import { gray } from "@twind/preset-radix-ui/colors";
import {
  gray as tailwindGray,
  green as tailwindGreen,
  red as tailwindRed,
  blue as tailwindBlue,
} from "@twind/preset-tailwind/colors";

if (process.env.MODE === "development") {
  setupTwind({
    tailwind: {
      colors: {
        brand: {
          1: "hsl(210deg,65%,99.5%)",
          2: "hsl(210deg,100%,99%)",
          3: "hsl(210deg,96.9%,97.4%)",
          4: "hsl(210deg,91.5%,95.5%)",
          5: "hsl(210deg,85.1%,93%)",
          6: "hsl(210deg,77.8%,89.4%)",
          7: "hsl(210deg,71%,83.7%)",
          8: "hsl(210deg,68.6%,76.3%)",
          9: "hsl(210deg,56%,57.5%)",
          10: "hsl(210deg,48.1%,53.5%)",
          11: "hsl(210deg,43%,48%)",
          12: "hsl(210deg,60%,18.5%)",
        },
        accent: gray,
        gray: tailwindGray,
        green: tailwindGreen,
        red: tailwindRed,
        blue: tailwindBlue,
      },
    },
  });
}

mount(
  () => (
    <AppContextProvider>
      <ClientRoot />
    </AppContextProvider>
  ),
  document
);
