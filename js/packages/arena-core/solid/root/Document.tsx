// credit: solid-start
import type { JSX } from "solid-js";
import { children, ComponentProps } from "solid-js";
import { insert, NoHydration, spread, ssrElement } from "solid-js/web";
import Links from "./Links";
import Meta from "./Meta";
import { Scripts } from "./Scripts";

const Html = (props: ComponentProps<"html">) => {
  if (process.env.SSR) {
    return ssrElement(
      "html",
      props,
      undefined,
      false
    ) as unknown as JSX.Element;
  }
  spread(document.documentElement, props, false, true);
  return props.children;
};

const Head = (props: ComponentProps<"head">) => {
  if (process.env.SSR) {
    return ssrElement(
      "head",
      props,
      () => (
        <>
          {props.children}
          <Meta />
          <Links />
        </>
      ),
      false
    ) as unknown as JSX.Element;
  } else {
    spread(document.head, props, false, true);
    return props.children;
  }
};

const Body = (props: ComponentProps<"body">) => {
  if (process.env.SSR) {
    return ssrElement(
      "body",
      props,
      () => (process.env.ARENA_SSR ? props.children : <Scripts />),
      false
    ) as unknown as JSX.Element;
  } else {
    if (process.env.ARENA_SSR) {
      let child = children(() => props.children);
      spread(document.body, props, false, true);
      insert(
        document.body,
        () => {
          let childNodes = child();
          if (childNodes) {
            if (Array.isArray(childNodes)) {
              let els = childNodes.filter((n) => Boolean(n));

              if (!els.length) {
                return null;
              }

              return els;
            }
            return childNodes;
          }
          return null;
        },
        null,
        [...document.body.childNodes]
      );

      return document.body;
    } else {
      spread(document.body, props, false, true);
      return props.children;
    }
  }
};

export { Html, Head, Body };
