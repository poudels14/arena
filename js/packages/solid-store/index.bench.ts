import { describe, bench, afterAll } from "vitest";
import { createStore } from ".";
import { createStore as createSolidStore } from "solid-js/store";
import { createRoot, createEffect, onCleanup } from "solid-js";

let arenaStoreRuns = 0;
let solidStoreRuns = 0;
afterAll(() => {
  console.log("Total @arena/solid-store runs =", arenaStoreRuns);
  console.log("Total solid-js/store runs =", solidStoreRuns);
});

describe("Store", () => {
  const TOTAL_UPDATES = 5_000;
  const TIME = 10;
  const ITERATIONS = 200;

  // bench(
  //   "[@arena/solid-store]: update simple value",
  //   () =>
  //     new Promise((done) => {
  //       createRoot(() => {
  //         const [store, setStore] = createStore({
  //           data: "Hello [at: 0]",
  //         });

  //         let count = 0;
  //         createEffect(() => {
  //           count += 1;
  //           void store.data;
  //           if (count < TOTAL_UPDATES) {
  //             setStore("data", "Hello [at: " + count + "]");
  //           } else {
  //             done();
  //           }
  //         });
  //       });
  //     }),
  //   { time: TIME, iterations: ITERATIONS }
  // );

  bench(
    "[@arena/solid-store]: update super nested value",
    () =>
      new Promise((done) => {
        createRoot(() => {
          const [store, setStore] = createStore({
            data: {
              group: {
                channel: {
                  message: {
                    text: {
                      value: "Hello [at: 0]",
                      counter: 0,
                    },
                  },
                },
              },
            },
          });

          createEffect(() => {
            void store.data.group.channel.message.text.value();
            const count = store.data.group.channel.message.text.counter();
            if (count < TOTAL_UPDATES) {
              setStore(
                "data",
                "group",
                "channel",
                "message",
                "text",
                "value",
                "Hello [at: " + count + "]"
              );
              setStore(
                "data",
                "group",
                "channel",
                "message",
                "text",
                "counter",
                count + 1
              );
            } else {
              done();
            }

            arenaStoreRuns += 1;
          });
        });
      }),
    { time: TIME, iterations: ITERATIONS }
  );

  bench(
    "[solidjs/store]: update super nested value",
    () =>
      new Promise((done) => {
        createRoot(() => {
          const [store, setStore] = createSolidStore({
            data: {
              group: {
                channel: {
                  message: {
                    text: {
                      value: "Hello [at: 0]",
                      counter: 0,
                    },
                  },
                },
              },
            },
          });

          createEffect(() => {
            const count = store.data.group.channel.message.text.counter;
            void store.data.group.channel.message.text.value;
            if (count < TOTAL_UPDATES) {
              setStore(
                "data",
                "group",
                "channel",
                "message",
                "text",
                "value",
                "Hello [at: " + count + "]"
              );
              setStore(
                "data",
                "group",
                "channel",
                "message",
                "text",
                "counter",
                count + 1
              );
            } else {
              done();
            }
            solidStoreRuns += 1;
          });
        });
      }),
    { time: TIME, iterations: ITERATIONS }
  );
});
