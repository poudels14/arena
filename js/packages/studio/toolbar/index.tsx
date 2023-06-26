import {
  createContext,
  JSX,
  useContext,
  Switch,
  Match,
  createComputed,
  createSelector,
  Accessor,
  lazy,
} from "solid-js";
import { A } from "@solidjs/router";
import { createStore, Store, StoreSetter } from "@arena/solid-store";
import DragHandle from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/drag-handle-horizontal";
import MinimizeIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/minimize";
import MaximizeIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/maximize";
import HomeIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/home";
import { InlineIcon } from "@arena/components";
import { Templates } from "./Templates";
import GeneralInfo from "./GeneralInfo";
import StyleEditor from "./StyleEditor";
import Data from "./Data";
import { ComponentTreeContext, useEditorContext } from "../editor";
import { ComponentTree } from "./ComponentTree";
// @ts-ignore

type ToolbarTab =
  /**
   * Show Arena AI chat box
   */
  | "chat"
  | "general"
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
    },
  });

  const isActive = createSelector(state.tab.active);
  const { getSelectedWidgets, setViewOnly, isViewOnly, getComponentTree } =
    useEditorContext<ComponentTreeContext>();
  const isWidgetHighlighted = () => getSelectedWidgets().length == 1;
  createComputed(() => {
    setState("tab", (prev) => {
      return {
        active: isWidgetHighlighted()
          ? prev.active != "templates"
            ? prev.active
            : "data"
          : "templates",
      };
    });
  });

  return (
    <ToolbarContext.Provider value={{ state, setState }}>
      <Switch>
        <Match when={isViewOnly()}>
          <div class="toolbar fixed bottom-0 w-full flex flex-row justify-center pointer-events-none z-[99999]">
            <div class="relative bottom-4 w-52 h-8 flex rounded-md text-brand-2 bg-brand-12/80 cursor-pointer pointer-events-auto space-x-2">
              <A href="/apps">
                <div class="inline-block px-2 py-2 rounded-l-md border-r border-brand-11/30 hover:bg-brand-10">
                  <InlineIcon size="16px" class="cursor-pointer">
                    <path d={HomeIcon[0]} />
                  </InlineIcon>
                </div>
              </A>

              <div
                class="flex-1 flex p-2 text-xs text-center"
                onClick={() => setViewOnly(false)}
              >
                <div class="flex-1">Open editor</div>
                <InlineIcon size="16px" class="py-1 cursor-pointer">
                  <path d={MaximizeIcon[0]} />
                </InlineIcon>
              </div>
            </div>
          </div>
        </Match>
        <Match when={true}>
          <div class="toolbar fixed bottom-0 w-full flex flex-row h-64 bg-brand-12/50 backdrop-blur justify-between pointer-events-auto z-[99999]">
            <div class="p-4 w-64 min-w-[200px] overflow-auto no-scrollbar hidden md:block">
              <ComponentTree node={getComponentTree()} />
            </div>
            <div class="flex basis-[840px] flex-col bg-brand-12/80">
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
              <div class="max-w-[840px] flex-1 px-2 overflow-x-auto no-scrollbar">
                <TabContent
                  isActive={isActive}
                  isWidgetHighlighted={isWidgetHighlighted}
                />
              </div>
              <ToolbarTabs
                isActive={isActive}
                disableWidgetConfigTabs={!isWidgetHighlighted()}
              />
            </div>
            <div class="w-64 hidden md:block"></div>
          </div>
        </Match>
      </Switch>
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
        hidden: props.disabled,
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
    <div class="px-4 py-1 flex flex-row space-x-2 text-sm text-brand-5 select-none">
      <A href="/apps">
        <div class="inline-block px-2 py-2 rounded-sm hover:bg-brand-12/80">
          <InlineIcon size="16px" class="cursor-pointer">
            <path d={HomeIcon[0]} />
          </InlineIcon>
        </div>
      </A>
      <div class="flex-1"></div>
      {/* <Tab
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
      </Tab> */}
      <Tab
        id="general"
        isActive={props.isActive}
        disabled={props.disableWidgetConfigTabs}
      >
        General
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

const TabContent = (props: {
  isActive: TabsProps["isActive"];
  isWidgetHighlighted: Accessor<boolean>;
}) => {
  return (
    <Switch>
      <Match when={props.isWidgetHighlighted() && props.isActive("general")}>
        <GeneralInfo />
      </Match>
      <Match when={props.isWidgetHighlighted() && props.isActive("data")}>
        <Data />
      </Match>
      <Match when={props.isWidgetHighlighted() && props.isActive("style")}>
        <StyleEditor />
      </Match>
      <Match when={props.isActive("templates")}>
        <Templates />
      </Match>
    </Switch>
  );
};

export { Toolbar };
