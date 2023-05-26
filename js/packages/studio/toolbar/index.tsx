import { createStore, Store, StoreSetter } from "@arena/solid-store";
import {
  createContext,
  JSX,
  useContext,
  Switch,
  Match,
  createMemo,
  createComputed,
  createEffect,
} from "solid-js";
import DragHandle from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/drag-handle-horizontal";
import MinimizeIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/minimize";
import MaximizeIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/maximize";
import { InlineIcon } from "@arena/components";
import { Templates } from "./Templates";
import { Data } from "./Data";
import { useEditorContext } from "../editor";

type ToolbarTab =
  /**
   * Show Arena AI chat box
   */
  | "chat"
  /**
   * Show data editor
   */
  | "data"
  /**
   * Show style editor
   */
  | "style"
  | "components"
  /**
   * Show templates and their preview
   */
  | "templates";

type ToolbarState = {
  collapsed: boolean;
  tab: {
    active: ToolbarTab;
    /**
     * Whether any widget is active; this is used to toggle widget
     * config tabs
     */
    isWidgetActive: boolean;
  };
};

const ToolbarContext = createContext<{
  state: Store<ToolbarState>;
  setState: StoreSetter<ToolbarState>;
}>();

const Toolbar = () => {
  const [state, setState] = createStore<ToolbarState>({
    collapsed: false,
    tab: {
      active: "templates",
      isWidgetActive: false,
    },
  });

  const { getSelectedWidgets } = useEditorContext();

  createComputed(() => {
    const selectedWidgets = getSelectedWidgets();
    setState("tab", (prev) => {
      const isWidgetActive = selectedWidgets.length == 1;
      return {
        active: isWidgetActive ? "data" : prev.active,
        isWidgetActive,
      };
    });
  });

  return (
    <ToolbarContext.Provider value={{ state, setState }}>
      <div class="toolbar-container fixed bottom-4 w-full flex flex-row justify-center pointer-events-none z-[99999]">
        <Switch>
          <Match when={state.collapsed()}>
            <div
              class="w-52 h-8 p-2 flex rounded-md text-gray-400 bg-slate-700 cursor-pointer pointer-events-auto space-x-2"
              onClick={() => setState("collapsed", false)}
            >
              <div class="flex-1 text-xs text-center">Open toolbar</div>
              <InlineIcon
                size="16px"
                class="py-1 cursor-pointer"
                onClick={() => setState("collapsed", true)}
              >
                <path d={MaximizeIcon[0]} />
              </InlineIcon>
            </div>
          </Match>
          <Match when={true}>
            <div class="toolbar flex flex-col w-[840px] h-64 rounded-md bg-slate-700 pointer-events-auto">
              <div class="relative py-0.5 flex justify-center text-white overflow-hidden">
                <InlineIcon size="14px" class="cursor-pointer">
                  <path d={DragHandle[0]} />
                </InlineIcon>
                <div class="absolute right-0 px-1">
                  <InlineIcon
                    size="12px"
                    class="p-[1px] cursor-pointer"
                    onClick={() => setState("collapsed", true)}
                  >
                    <path d={MinimizeIcon[0]} />
                  </InlineIcon>
                </div>
              </div>
              <div class="flex-1 px-2 overflow-hidden">
                <TabContent activeTab={state.tab.active()} />
              </div>
              <ToolbarTabs
                active={state.tab.active()}
                disableWidgetConfigTabs={!state.tab.isWidgetActive()}
              />
            </div>
          </Match>
        </Switch>
      </div>
    </ToolbarContext.Provider>
  );
};

type TabsProps = {
  id: ToolbarTab;
  children: JSX.Element;
  active: ToolbarTab;
  classList?: Record<string, boolean>;
  disabled?: boolean;
};

const Tab = (props: TabsProps) => {
  const { setState } = useContext(ToolbarContext)!;
  return (
    <div
      class="px-4 py-0.5 my-auto rounded"
      classList={{
        ...(props.classList || {}),
        "cursor-pointer": !props.disabled,
        "text-white bg-slate-600": props.active === props.id,
        "text-gray-500 cursor-not-allowed": props.disabled,
      }}
      onClick={() => !props.disabled && setState("tab", "active", props.id)}
    >
      {props.children}
    </div>
  );
};

const ToolbarTabs = (props: {
  active: ToolbarTab;
  disableWidgetConfigTabs: boolean;
}) => {
  return (
    <div class="px-4 py-1 flex flex-row space-x-2 text-sm text-gray-400 select-none">
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
      <Tab
        id="data"
        active={props.active}
        disabled={props.disableWidgetConfigTabs}
      >
        Data
      </Tab>
      <Tab
        id="style"
        active={props.active}
        disabled={props.disableWidgetConfigTabs}
      >
        Style
      </Tab>
      <Tab id="templates" active={props.active}>
        Templates
      </Tab>
    </div>
  );
};

const TabContent = (props: { activeTab: ToolbarTab }) => {
  return (
    <Switch>
      <Match when={props.activeTab == "data"}>
        <Data />
      </Match>
      <Match when={props.activeTab == "templates"}>
        <Templates />
      </Match>
    </Switch>
  );
};

export { Toolbar };
