import {
  createStore,
  Store,
  StoreSetter,
  StoreValue,
} from "@arena/solid-store";

type App = {
  id: string;
  name: string;
  description?: string;
};

type Node = StoreValue<{
  title: string;
  icon?: string;
  children: StoreValue<Node[]>;
}>;

class EditorState {
  #store: Store<{ app: any }>;
  #setStore: StoreSetter<any>;
  constructor() {
    // TODO(sagar): use sync store instead
    const s = createStore({
      app: {
        mode: "edit",
        id: "app1",
        name: "My first app!",
        description: "A description for my new app",
        components: [],
      },
    });
    // @ts-ignore
    this.#store = s[0];
    this.#setStore = s[1];
  }

  getApp() {
    return this.#store.app;
  }

  getComponentTree() {
    return {
      title: this.getApp().name,
      children: () => [],
    } as unknown as StoreValue<Node>;
  }
}

export { EditorState };
export type { App, Node };
