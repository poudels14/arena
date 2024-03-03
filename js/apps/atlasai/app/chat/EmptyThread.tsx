import { useSharedWorkspaceContext } from "@portal/workspace-sdk";
import { ChatQueryContext } from "./ChatContext";
import { For, Show, createComputed, untrack } from "solid-js";
import { createSyncedStore } from "@portal/solid-store";

type EmptyThreadProps = {
  contextSelection?: ChatQueryContext;
};

const EmptyThread = (props: EmptyThreadProps) => {
  const { setChatContext } = useSharedWorkspaceContext();
  const [state, setState] = createSyncedStore<{ selectedContext: any[] }>(
    {
      selectedContext: [],
    },
    {
      storeKey: "atlasai/chat/emptythread/state",
    }
  );
  createComputed(() => {
    const context = props.contextSelection || [];
    const prevSelectedContext = untrack(() => state.selectedContext());
    // if none of the previous context is selected,
    // dont select any new context either
    if (!prevSelectedContext.find((c) => c)) {
      return;
    }
    setState(
      "selectedContext",
      [...Array(context.length)].map(() => true)
    );
  });

  createComputed(() => {
    const selected = state.selectedContext();
    const context = props.contextSelection;
    if (!context) {
      return;
    }
    const filteredContext = context.filter((_, idx) => selected[idx]);
    setChatContext(filteredContext);
  });

  return (
    <div class="h-[calc(100%-theme(spacing.32))] py-16 flex flex-col justify-center space-y-8">
      <div class="font-bold text-xl text-gray-700 text-center">
        How can I help you?
      </div>

      <Show when={props.contextSelection}>
        <div class="flex justify-center">
          <div class="basis-60 space-y-3">
            <div class="text-md font-bold text-gray-600">Search</div>
            <div>
              <For each={props.contextSelection}>
                {(context, index) => {
                  return (
                    <label class="px-4 py-2 flex text-sm space-x-2 rounded-md border border-gray-100 has-[:checked]:bg-indigo-50 has-[:checked]:border-indigo-200">
                      <input
                        type="checkbox"
                        checked={state.selectedContext()[index()]}
                        onChange={() => {
                          setState("selectedContext", (prev) => {
                            const x = [...prev];
                            x[index()] = !state.selectedContext()[index()];
                            return x;
                          });
                        }}
                      />
                      <div>{context.app.name}</div>
                    </label>
                  );
                }}
              </For>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export { EmptyThread };
