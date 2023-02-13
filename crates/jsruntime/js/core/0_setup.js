"use strict";

Deno.core.initializeAsyncOps();
const { performance } = globalThis.__bootstrap.performance;
const {
  setTimeout,
  clearTimeout,
  setInterval,
  clearInterval,
  handleTimerMacrotask,
} = globalThis.__bootstrap.timers;
Deno.core.setMacrotaskCallback(handleTimerMacrotask);
const { Request, Response } = globalThis.__bootstrap.fetch;
const { ReadableStream } = globalThis.__bootstrap.streams;
const { crypto } = globalThis.__bootstrap.crypto;

const {
  TextEncoder,
  TextDecoder,
  TextEncoderStream,
  TextDecoderStream,
  encode,
  decode,
} = globalThis.__bootstrap.encoding;

// Note(sagar): assign to globalThis so that other modules can access
// these objects with `globalThis.{}`
((globalThis) => {
  Object.assign(globalThis, {
    crypto,
    setTimeout,
    clearTimeout,
    setInterval,
    clearInterval,
    performance,
    Request,
    Response,
    ReadableStream,
    TextEncoder,
    TextDecoder,
    TextEncoderStream,
    TextDecoderStream,
  });
})(globalThis);
