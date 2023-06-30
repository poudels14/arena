import { createEffect } from "solid-js";
import { createStore } from "./core";

/**
 * Creates a store that's synced with local storage
 */
function createSyncedStore<T>(initValue: T, options: { storeKey: string }) {
  const cache = localStorage.getItem(options.storeKey);
  if (cache) {
    initValue = JSON.parse(cache);
  }
  const [store, setStore] = createStore<T>(initValue);
  createEffect(() => {
    localStorage.setItem(options.storeKey, JSON.stringify(store()));
  });
  return [store, setStore] as ReturnType<typeof createStore<T>>;
}

export { createSyncedStore };
