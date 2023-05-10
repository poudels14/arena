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
import { Console } from "ext:deno_console/02_console.js";

const primordials = globalThis.__bootstrap.primordials;
const {
  ArrayPrototypePush,
  ArrayPrototypeIndexOf,
  ArrayPrototypeSplice,
  WeakMapPrototypeSet,
  WeakMapPrototypeDelete,
  DateNow,
} = primordials;

// credit: deno
function promiseRejectCallback(type, promise, reason) {
  console.log("promiseRejectCallback called: ", arguments);
  switch (type) {
    case 0: {
      ops.op_store_pending_promise_rejection(promise, reason);
      ArrayPrototypePush(pendingRejections, promise);
      WeakMapPrototypeSet(pendingRejectionsReasons, promise, reason);
      break;
    }
    case 1: {
      ops.op_remove_pending_promise_rejection(promise);
      const index = ArrayPrototypeIndexOf(pendingRejections, promise);
      if (index > -1) {
        ArrayPrototypeSplice(pendingRejections, index, 1);
        WeakMapPrototypeDelete(pendingRejectionsReasons, promise);
      }
      break;
    }
    default:
      return false;
  }

  return (
    !!globalThis_.onunhandledrejection ||
    event.listenerCount(globalThis_, "unhandledrejection") > 0
  );
}

// Note(sagar): this is initialized during snapshotting
// assign to globalThis so that other modules can access
// these objects with `globalThis.{}`
((globalThis) => {
  const { core } = Deno;
  setTimeOrigin(DateNow());

  core.setPromiseRejectCallback(promiseRejectCallback);

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
    performance,
    ReadableStream,
    TextEncoder,
    TextDecoder,
    TextEncoderStream,
    TextDecoderStream,
    encodeToBase64,
    encodeToBase64Url,
  });
})(globalThis);
