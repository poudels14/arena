import { ReadableStream } from "ext:deno_web/06_streams.js";
import { Headers } from "ext:deno_fetch/20_headers.js";
import { Request } from "ext:deno_fetch/23_request.js";
import { Response, toInnerResponse } from "ext:deno_fetch/23_response.js";
import { URL, URLSearchParams } from "ext:deno_url/00_url.js";

// Note(sagar): this is initialized during snapshotting
// assign to globalThis so that other modules can access
// these objects with `globalThis.{}`
((globalThis) => {
  Response.toInnerResponse = toInnerResponse;

  Object.assign(globalThis, {
    Headers,
    Request,
    Response,
    ReadableStream,
    URL,
    URLSearchParams,
  });
})(globalThis);
