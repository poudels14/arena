// credit: solid-start
import { JSX } from "solid-js";
import { hydrate, render } from "solid-js/web";
import env from "../env";

const mount = (code?: () => JSX.Element, element?: Document) => {
  if (env.ARENA_SSR) {
    code && element && hydrate(code, element);
  } else {
    code &&
      element &&
      render(code, element === document ? element.body : element);
  }
};

export { mount };
