import { describe, expect, test, vitest } from "vitest";
import { createStore, $RAW } from ".";
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

  test("Update nested store value - parent value should be reactive", () =>
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

  test("Update nested object to undefined - rest empty root", () =>
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
});
