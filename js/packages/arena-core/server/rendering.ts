import { JSX } from "solid-js";
import { renderToStringAsync } from "solid-js/web";

type Fn<R, Args extends Array<any>> = (...a: Args) => R;

function renderAsync<A1>(
  cb: Fn<JSX.Element, [A1]>
): Fn<Promise<Response>, [A1]>;
function renderAsync<A1, A2>(
  cb: Fn<JSX.Element, [A1, A2]>
): Fn<Promise<Response>, [A1, A2]>;
function renderAsync<A1, A2, A3>(
  cb: Fn<JSX.Element, [A1, A2, A3]>
): Fn<Promise<Response>, [A1, A2, A3]>;

function renderAsync<A1, A2, A3, A4>(fn: Fn<JSX.Element, [A1, A2, A3, A4]>) {
  return async (...args: [A1, A2, A3, A4]) => {
    const html = await renderToStringAsync(() => fn(...args));

    return new Response(html, {
      status: 200,
      headers: {
        "content-type": "text/html; charset=utf-8",
      },
    });
  };
}

export { renderAsync };
