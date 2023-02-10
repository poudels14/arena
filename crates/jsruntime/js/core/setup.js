"use strict";

Deno.core.initializeAsyncOps();
const { setTimeout, clearTimeout, setInterval, clearInterval, handleTimerMacrotask } = globalThis.__bootstrap.timers;
Deno.core.setMacrotaskCallback(handleTimerMacrotask);
const { Request, Response } = globalThis.__bootstrap.fetch;
const { ReadableStream } = globalThis.__bootstrap.streams;