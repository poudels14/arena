import { Show, createSelector } from "solid-js";
import { createStore } from "@arena/solid-store";
import { Documents } from "./Documents";

const Sidebar = () => {
  const [sidebarState, setState] = createStore({
    tab: {
      selected: "Chat",
    },
  });

  const isTabActive = createSelector(sidebarState.tab.selected);

  return (
    <div class="w-52 h-screen text-sm bg-accent-1">
      <div class="p-2">
        <SidebarTabs
          isTabActive={isTabActive}
          setSelected={(tab) => setState("tab", "selected", tab)}
        />
      </div>
      <div class="px-1">
        <Show when={isTabActive("Chat")}>
          <div class="space-y-4">
            <div>
              <div class="py-1 px-2 font-medium rounded cursor-pointer bg-brand-10/20 text-accent-12/70">
                Chat
              </div>
            </div>
            <div class="space-y-2">
              <Documents />
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};

const SidebarTabs = (props: {
  isTabActive: (id: string) => boolean;
  setSelected: (id: string) => void;
}) => {
  return (
    <div class="flex p-[3px] text-center rounded-lg space-x-2 text-xs bg-brand-11/20">
      <SidebarTab
        name="Chat"
        selected={props.isTabActive("Chat")}
        setSelected={props.setSelected}
      />
      <SidebarTab
        name="History"
        selected={props.isTabActive("History")}
        setSelected={props.setSelected}
      />
    </div>
  );
};

const SidebarTab = (props: {
  name: string;
  selected: boolean;
  setSelected: (id: string) => void;
}) => {
  return (
    <div
      class="flex-1 py-0.5 text-center cursor-pointer"
      classList={{
        "rounded-md text-accent-12/80 bg-white": props.selected,
        "text-accent-11": !props.selected,
      }}
      onClick={() => props.setSelected(props.name)}
    >
      {props.name}
    </div>
  );
};

export { Sidebar };
