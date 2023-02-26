// credit: solid-start
import { JSX } from "solid-js";
import { hydrate, render } from "solid-js/web";

export default function mount(code?: () => JSX.Element, element?: Document) {
  if (Arena.env.ARENA_SSR) {
    code && element && hydrate(code, element);
  } else {
    code &&
      element &&
      render(code, element === document ? element.body : element);
  }
}
