import { JSX } from "solid-js";
import { renderToStringAsync } from "solid-js/web";
import { PageEvent } from "./event";

const renderAsync = (fn: (ev: PageEvent) => JSX.Element) => {
  return async (event: PageEvent) => {
    const html = await renderToStringAsync(() => fn(event));

    return new Response(html, {
      // TODO(sagar): fix status code and headers
      status: 200,
      headers: {
        "content-type": "html",
      },
    });
  };
};

export { renderAsync };
