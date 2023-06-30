import { describe, expect, test, vitest } from "vitest";
import { createStore, $RAW } from "./core";
import { createRoot, createEffect, onCleanup } from "solid-js";

describe("Store", () => {
  test("create new store", () =>
    new Promise((done) => {
      const [store] = createStore({
        name: "Arena",
        data: {
          message: "Hello!",
        },
      });
      expect(store.data.message()).toBe("Hello!");
      done(null);
    }));

  test("Update simple nested store value", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            message: "Hello!",
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data.message();
          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore("data", "message", "hello, new world!");

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data.message()).toBe("hello, new world!");
            done(null);
          });
        });
      });
    }));

  test("Update null field with non-null value", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore<{ data: { message: string } }>({
          data: null!,
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data.message();
          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore("data", { message: "hello, new world!" });

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data.message!()).toBe("hello, new world!");
            done(null);
          });
        });
      });
    }));

  test("Using singal inside getter works", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            name: "World",
            get message() {
              // @ts-ignore
              return "Hello, " + this.name();
            },
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data.message();
          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore("data", "name", "Earth");

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data.message()).toBe("Hello, Earth");
            done(null);
          });
        });
      });
    }));

  test("Updating field with getter shouldn't call getter during update", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            text: "some text",
            get message() {
              return "Hello, World ";
            },
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data.text();
          onCleanup(() => cleanupFn());
        });

        const messagedAccessed = vitest.fn();
        setTimeout(() => {
          setStore("data", {
            text: "some text",
            get message() {
              return "Hello, Earth";
            },
          });

          setTimeout(() => {
            expect(messagedAccessed).toBeCalledTimes(0);
            // Note: since same text is passed, cleanup shouldn't run
            expect(cleanupFn).toBeCalledTimes(0);
            expect(store.data.message()).toBe("Hello, Earth");
            done(null);
          });
        });
      });
    }));

  test("Update nested store - parent value should be reactive", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            message: "Hello!",
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data();
          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore("data", "message", "hello, new world!");

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data[$RAW]).toStrictEqual({
              message: "hello, new world!",
            });
            done(null);
          });
        });
      });
    }));

  /**
   * This checks whether updating a leaf level field breaks when only
   * the higher level field is being listened to.
   */
  test("Update 3+ level nested store - parent value should be reactive", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            message: {
              text: {
                value: "Hello!",
              },
            },
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          // Note: only listen to store.data
          void store.data.message();
          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore("data", "message", "text", "value", "hello, new world!");

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data[$RAW]).toStrictEqual({
              message: {
                text: {
                  value: "hello, new world!",
                },
              },
            });
            done(null);
          });
        });
      });
    }));

  test("Update nested object to undefined - simple", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            message: "Hello!",
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data.message();
          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore("data", undefined!);

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data.message()).toBe(undefined);
            done(null);
          });
        });
      });
    }));

  test("Update nested object to undefined - multiple fields", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            timestamp: 11111,
            message: "Hello!",
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data.message();
          void store.data.timestamp();

          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore("data", undefined!);

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data.message()).toBe(undefined);
            expect(store.data.timestamp()).toBe(undefined);
            done(null);
          });
        });
      });
    }));

  test("Update nested object - set store value to empty object", () =>
    new Promise((done) => {
      createRoot(() => {
        const [store, setStore] = createStore({
          data: {
            message: "Hello!",
          },
        });

        const cleanupFn = vitest.fn();
        createEffect(() => {
          void store.data.message();
          onCleanup(() => cleanupFn());
        });

        setTimeout(() => {
          setStore({});

          setTimeout(() => {
            expect(cleanupFn).toBeCalledTimes(1);
            expect(store.data.message()).toBe(undefined);
            done(null);
          });
        });
      });
    }));

  // TODO(sagar): test a case where updating an object doesn't update and
  // notify it's sub fields if subfields aren't being used in reactive context
});
