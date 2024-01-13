import * as webidl from "ext:deno_webidl/00_webidl.js";
import { AbortSignal, AbortController } from "ext:deno_web/03_abort_signal.js";
import { performance, setTimeOrigin } from "ext:deno_web/15_performance.js";
import * as _ from "ext:deno_fetch/27_eventsource.js";
import * as globalInterfaces from "ext:deno_web/04_global_interfaces.js";
import {
  setTimeoutUnclamped,
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

function nonEnumerable(value) {
  return {
    value,
    writable: true,
    enumerable: false,
    configurable: true,
  };
}

// Note(sagar): this is initialized during snapshotting
// assign to globalThis so that other modules can access
// these objects with `globalThis.{}`
((globalThis) => {
  function setImmediate(cb, ...args) {
    return setTimeoutUnclamped(cb, 0, ...args);
  }

  Object.assign(globalThis, {
    __bootstrap: {
      ...globalThis.__bootstrap,
      handleTimerMacrotask,
      Console,
    },
    setImmediate,
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

    Event: event.Event,
    EventTarget: event.EventTarget,
    Window: globalInterfaces.Window,
  });

  Object.defineProperties(globalThis, {
    [webidl.brand]: nonEnumerable(webidl.brand),
  });
})(globalThis);

// credit: deno
function processUnhandledPromiseRejection(type, promise, reason) {
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

globalThis.__setupRuntime = () => {
  const primordials = globalThis.__bootstrap.primordials;
  const { DateNow } = primordials;

  const { core } = Deno;
  setTimeOrigin(DateNow());

  event.setEventTargetData(globalThis);
  event.saveGlobalThisReference(globalThis);

  Object.setPrototypeOf(globalThis, Window.prototype);

  core.setUnhandledPromiseRejectionHandler(processUnhandledPromiseRejection);
};
