import { useSharedWorkspaceContext } from "@portal/workspace-sdk";
import { ChatContext, ChatQueryContext } from "./ChatContext";
import {
  For,
  Show,
  createComputed,
  createMemo,
  createSelector,
  untrack,
  useContext,
} from "solid-js";
import { createSyncedStore } from "@portal/solid-store";
import { Select } from "@portal/solid-ui/form";

type EmptyThreadProps = {
  contextSelection?: ChatQueryContext;
};

const EmptyThread = (props: EmptyThreadProps) => {
  const { setChatContext, activeWorkspace, setChatConfig } =
    useSharedWorkspaceContext();
  const { getChatProfiles } = useContext(ChatContext)!;
  const [state, setState] = createSyncedStore<{
    defaultModel: string;
    selectedContext: any[];
    selectedChatProfile: string | null;
  }>(
    {
      defaultModel: "openai-gpt-3.5",
      selectedContext: [],
      selectedChatProfile: null,
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

  createComputed(() => {
    const defaultModel = state.defaultModel();
    const availableModels = activeWorkspace.models();
    const modelToUse =
      availableModels.find((m) => m.id == defaultModel) || availableModels[0];
    setChatConfig("model", modelToUse?.id);
  });

  const selectedProfileId = createMemo(() => {
    const selectedChatProfile = state.selectedChatProfile();
    const profiles = getChatProfiles();
    const selectedProfile =
      profiles.find((p) => p.id == selectedChatProfile)?.id ||
      profiles.find((p) => p.default)?.id ||
      profiles[0]?.id;
    setChatConfig("selectedProfileId", selectedProfile);
    return selectedProfile;
  });
  const isChatProfileSelected = createSelector(selectedProfileId);

  return (
    <div class="h-[calc(100%-theme(spacing.32))] py-16 flex flex-col justify-center space-y-8">
      <div class="font-bold text-xl text-gray-700 text-center">
        How can I help you?
      </div>

      <div class="flex justify-center space-x-6">
        <div class="px-4">
          <div class="basis-60 space-y-3">
            <div>
              <div class="text-md font-bold text-gray-600">AI Model</div>
              <div class="text-xs text-gray-400">
                Enable more models from Settings
              </div>
            </div>
            <div>
              <Select
                name="model"
                triggerClass="px-4 py-1.5 w-full text-sm"
                contentClass="w-full text-sm"
                itemClass="px-4 py-2 text-xs cursor-pointer hover:bg-gray-100"
                itemRenderer={(item) => {
                  return (
                    <div class="flex-1 flex justify-between">
                      <div>{item.rawValue.name}</div>
                      <Show when={item.rawValue.disabled}>
                        <div class="text-red-700">Disabled</div>
                      </Show>
                    </div>
                  );
                }}
                placeholder="Select Model"
                options={activeWorkspace.models().filter((m) => !m.disabled)}
                optionValue="id"
                optionTextValue="name"
                optionDisabled="disabled"
                value={state.defaultModel()}
                onChange={(value) => {
                  setState("defaultModel", value);
                }}
              />
            </div>
            <Show when={props.contextSelection}>
              <div class="flex justify-center">
                <div class="basis-60 space-y-3">
                  <div class="text-md font-bold text-gray-600">Search</div>
                  <div>
                    <For each={props.contextSelection}>
                      {(context, index) => {
                        return (
                          <label class="px-4 py-2 flex text-sm items-center space-x-2 rounded-md border border-gray-100 has-[:checked]:bg-indigo-50 has-[:checked]:border-indigo-200">
                            <input
                              type="checkbox"
                              checked={state.selectedContext()[index()]}
                              onChange={() => {
                                setState("selectedContext", (prev) => {
                                  const x = [...prev];
                                  x[index()] =
                                    !state.selectedContext()[index()];
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
        </div>
        <div class="basis-60 py-2 px-4 space-y-3">
          <Show when={getChatProfiles().length > 0}>
            <div class="text-md font-bold text-gray-600">
              Select chat profile
            </div>
            <div class="max-h-48 text-xs rounded border border-gray-200 divide-y divide-gray-100 overflow-auto scroll:w-[2px] thumb:rounded thumb:bg-gray-200">
              <For each={getChatProfiles()}>
                {(profile) => {
                  return (
                    <div
                      class="px-2 py-1.5 border-y cursor-pointer first:border-t-0 last:border-b-0"
                      classList={{
                        "bg-indigo-50": isChatProfileSelected(profile.id),
                        "border-transparent": !isChatProfileSelected(
                          profile.id
                        ),
                      }}
                      onClick={() => {
                        setState("selectedChatProfile", profile.id);
                      }}
                    >
                      {profile.name}
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export { EmptyThread };
