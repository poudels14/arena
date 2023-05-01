// TODO(sagar): make dispatchEvent, etc work
//
// import { EventTarget } from "internal:deno_web/02_event.js";

// console.log(globalThis.__bootstrap.event);

// const { EventTarget } = globalThis.__bootstrap.event;

// console.log(EventTarget)

// const windowDispatchEvent = EventTarget.prototype.dispatchEvent.bind(
//   globalThis,
// );

// dispatchEvent(new Event("YO!!!"));

globalThis.addEventListener("load", () => {
  console.log("LOADED!");
});
