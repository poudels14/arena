import { createStore } from "@arena/solid-store";
import { klona } from "klona";
import type { BaseConfig, CoreState, InternalEditor } from "./types";

function createBaseEditor<S, M>(config: BaseConfig) {
  config = klona(config);
  // @ts-expect-error
  const [coreState, setCoreState] = createStore<CoreState>({});

  const [pluginsState, setPluginsState] = createStore<any>({});

  const editor = {
    context: {
      state: coreState,
    },
    components: [],
    core: {
      config,
      state: coreState,
      setState: setCoreState,
    },
    plugins: {
      state: pluginsState,
      setState: setPluginsState,
    },
  } as unknown as InternalEditor<S, M>;
  return editor;
}

export { createBaseEditor };
