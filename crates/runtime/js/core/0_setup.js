import { AbortSignal, AbortController } from "ext:deno_web/03_abort_signal.js";
import { performance, setTimeOrigin } from "ext:deno_web/15_performance.js";
import {
  setTimeout,
  clearTimeout,
  setInterval,
  clearInterval,
  handleTimerMacrotask,
} from "ext:deno_web/02_timers.js";
import * as event from "ext:deno_web/02_event.js";
import {
  TextEncoder,
  TextDecoder,
  TextEncoderStream,
  TextDecoderStream,
} from "ext:deno_web/08_text_encoding.js";
import {
  forgivingBase64Encode as encodeToBase64,
  forgivingBase64UrlEncode as encodeToBase64Url,
} from "ext:deno_web/00_infra.js";
import { ReadableStream } from "ext:deno_web/06_streams.js";
import { fetch } from "ext:deno_fetch/26_fetch.js";
import { Console } from "ext:deno_console/01_console.js";

const primordials = globalThis.__bootstrap.primordials;
const { DateNow } = primordials;

// credit: deno
function promiseRejectCallback(type, promise, reason) {
  console.log("PROMISE REJECTED! type:", type, "reason:", reason);
  const rejectionEvent = new event.PromiseRejectionEvent("unhandledrejection", {
    cancelable: true,
    promise,
    reason,
  });

  // Note that the handler may throw, causing a recursive "error" event
  globalThis.dispatchEvent(rejectionEvent);
  // TODO: there is more to this
}

// Note(sagar): this is initialized during snapshotting
// assign to globalThis so that other modules can access
// these objects with `globalThis.{}`
((globalThis) => {
  const { core } = Deno;
  setTimeOrigin(DateNow());

  core.setUnhandledPromiseRejectionHandler(promiseRejectCallback);

  event.setEventTargetData(globalThis);
  event.saveGlobalThisReference(globalThis);

  Object.assign(globalThis, {
    __bootstrap: {
      ...globalThis.__bootstrap,
      handleTimerMacrotask,
      Console,
    },
    setTimeout,
    clearTimeout,
    setInterval,
    clearInterval,
    AbortSignal,
    AbortController,
    performance,
    ReadableStream,
    TextEncoder,
    TextDecoder,
    TextEncoderStream,
    TextDecoderStream,
    fetch,
    encodeToBase64,
    encodeToBase64Url,
  });
})(globalThis);
