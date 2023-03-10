import {
  createStore,
  Store,
  StoreValue,
  StoreSetter,
} from "@arena/solid-store";
import { createContext, JSX, useContext } from "solid-js";

type ToolbarTab = "chat" | "data" | "style" | "components" | "templates";
type TabsProps = {
  id: ToolbarTab;
  children: JSX.Element;
  active: StoreValue<ToolbarTab>;
  classList?: Record<string, boolean>;
};

const Tab = (props: TabsProps) => {
  const { setStore } = useContext(ToolbarContext)!;
  return (
    <div
      class="px-4 rounded cursor-pointer"
      classList={{
        ...(props.classList || {}),
        "text-white bg-slate-500": props.active() === props.id,
      }}
      onMouseDown={() => setStore("tab", "active", props.id)}
    >
      {props.children}
    </div>
  );
};

const ToolbarTabs = (props: { active: TabsProps["active"] }) => {
  return (
    <div class="px-4 py-1 h-8 flex flex-row space-x-2 text-gray-400 select-none">
      <Tab
        id="chat"
        active={props.active}
        classList={{
          "flex-1": true,
        }}
      >
        <input
          type="text"
          placeholder="Ask Arena..."
          class="w-full py-1 text-sm placeholder:text-gray-400 bg-transparent text-white rounded-l outline-none"
        />
      </Tab>
      <Tab id="data" active={props.active}>
        Data
      </Tab>
      <Tab id="style" active={props.active}>
        Style
      </Tab>
      <Tab id="components" active={props.active}>
        Components
      </Tab>
      <Tab id="templates" active={props.active}>
        Templates
      </Tab>
    </div>
  );
};

type ToolbarState = {
  tab: {
    active: ToolbarTab;
  };
};

const ToolbarContext = createContext<{
  store: Store<ToolbarState>;
  setStore: StoreSetter<ToolbarState>;
}>();

const Toolbar = () => {
  const [store, setStore] = createStore<ToolbarState>({
    tab: {
      active: "data",
    },
  });

  return (
    <ToolbarContext.Provider value={{ store, setStore }}>
      {/* // TODO(sagar): make the element behind this toolbar clickable */}
      <div class="absolute bottom-4 w-full flex flex-row justify-center z-[10000]">
        <div class="flex flex-col w-[840px] h-52 rounded-md bg-slate-700">
          <div class="flex-1"></div>
          <ToolbarTabs active={store.tab.active} />
        </div>
      </div>
    </ToolbarContext.Provider>
  );
};

export { Toolbar };
