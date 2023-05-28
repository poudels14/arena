import { createStore, Store, StoreSetter } from "@arena/solid-store";
import {
  createContext,
  JSX,
  useContext,
  Switch,
  Match,
  createComputed,
  createSelector,
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
    tab: {
      active: "templates",
      isWidgetActive: false,
    },
  });

  const isActive = createSelector(state.tab.active);
  const { getSelectedWidgets, setViewOnly, isViewOnly } = useEditorContext();

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
          <Match when={isViewOnly()}>
            <div
              class="w-52 h-8 p-2 flex rounded-md text-brand-2 bg-brand-12/80 cursor-pointer pointer-events-auto space-x-2"
              onClick={() => setViewOnly(false)}
            >
              <div class="flex-1 text-xs text-center">Open editor</div>
              <InlineIcon size="16px" class="py-1 cursor-pointer">
                <path d={MaximizeIcon[0]} />
              </InlineIcon>
            </div>
          </Match>
          <Match when={true}>
            <div class="toolbar flex flex-col w-[840px] h-64 rounded-md bg-brand-12/80 backdrop-blur-2xl pointer-events-auto">
              <div class="relative py-0.5 flex justify-center text-white overflow-hidden">
                <InlineIcon size="14px" class="cursor-pointer">
                  <path d={DragHandle[0]} />
                </InlineIcon>
                <div class="absolute right-0 px-1">
                  <InlineIcon
                    size="14px"
                    class="p-[2px] cursor-pointer"
                    onClick={() => setViewOnly(true)}
                  >
                    <path d={MinimizeIcon[0]} />
                  </InlineIcon>
                </div>
              </div>
              <div class="flex-1 px-2 overflow-hidden">
                <TabContent isActive={isActive} />
              </div>
              <ToolbarTabs
                isActive={isActive}
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
  isActive: (id: ToolbarTab) => boolean;
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
        "bg-brand-11 text-brand-1": props.isActive(props.id),
        "text-brand-11 cursor-not-allowed": props.disabled,
      }}
      onClick={() => !props.disabled && setState("tab", "active", props.id)}
    >
      {props.children}
    </div>
  );
};

const ToolbarTabs = (props: {
  isActive: TabsProps["isActive"];
  disableWidgetConfigTabs: boolean;
}) => {
  return (
    <div class="px-4 py-1 flex flex-row space-x-2 text-sm text-brand-7 select-none">
      <Tab
        id="chat"
        isActive={props.isActive}
        classList={{
          "flex-1": true,
        }}
      >
        <input
          type="text"
          placeholder="Ask Arena..."
          class="w-full py-1 text-sm placeholder:text-brand-5 bg-transparent text-brand-1 rounded-l outline-none"
        />
      </Tab>
      <Tab
        id="data"
        isActive={props.isActive}
        disabled={props.disableWidgetConfigTabs}
      >
        Data
      </Tab>
      <Tab
        id="style"
        isActive={props.isActive}
        disabled={props.disableWidgetConfigTabs}
      >
        Style
      </Tab>
      <Tab id="templates" isActive={props.isActive}>
        Templates
      </Tab>
    </div>
  );
};

const TabContent = (props: { isActive: TabsProps["isActive"] }) => {
  return (
    <Switch>
      <Match when={props.isActive("data")}>
        <Data />
      </Match>
      <Match when={props.isActive("templates")}>
        <Templates />
      </Match>
    </Switch>
  );
};

export { Toolbar };
