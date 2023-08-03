import { For, Show, createSelector } from "solid-js";
import { createStore } from "@arena/solid-store";
import { Document } from "./types";

type SidebarProps = {
  documents: Document[];
};

const Sidebar = (props: SidebarProps) => {
  const [state, setState] = createStore({
    tab: {
      selected: "Chat",
    },
  });

  const isTabActive = createSelector(state.tab.selected);

  return (
    <div class="w-52 h-screen text-sm bg-accent-2">
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
              <div class="py-1 px-2 font-medium rounded cursor-pointer bg-brand-12/20 text-brand-12">
                Chat
              </div>
            </div>
            <div class="space-y-2">
              <div class="px-2 text-base font-medium text-gray-800">
                Linked Documents
              </div>
              <div>
                <For each={props.documents}>
                  {(document) => (
                    <Document id={document.id} name={document.filename} />
                  )}
                </For>
              </div>
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
    <div class="flex p-[3px] text-center rounded-lg space-x-2 text-xs bg-brand-12/60">
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
        "text-white": !props.selected,
      }}
      onClick={() => props.setSelected(props.name)}
    >
      {props.name}
    </div>
  );
};

const Document = (props: { id: string; name: string; active?: boolean }) => {
  return (
    <label class="flex align-middle items-center">
      <div class="group relative">
        {/* <input
          type="checkbox"
          checked={props.active}
          disabled
          class="peer/check w-5 h-5 opacity-0 hidden cursor-pointer border border-red-200"
        /> */}
        {/* <div class="relative w-3 h-3 rounded bg-brand-5 peer-checked/check:bg-brand-11">
          <div class="absolute top-px left-1 w-1 h-2 border-gray-100 border-l-0 border-t-0 border-b-2 border-r-2 rotate-45"></div>
        </div> */}
      </div>
      <div
        class="flex-1 py-0.5 px-2 rounded cursor-pointer text-accent-12/90 hover:bg-accent-4"
        classList={
          {
            // "text-accent-9": !props.active,
          }
        }
      >
        {props.name}
      </div>
    </label>
  );
};

export { Sidebar };
