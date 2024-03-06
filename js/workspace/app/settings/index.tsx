import { Show, For } from "solid-js";
import { createMutationQuery, createQuery } from "@portal/solid-query";
import { useSharedWorkspaceContext, Workspace } from "@portal/workspace-sdk";
import { HiOutlineTrash } from "solid-icons/hi";

const WorkspaceSettings = () => {
  const { activeWorkspace } = useSharedWorkspaceContext();
  const settingsQuery = createQuery<{ models: Workspace.Model[] }>(
    () => `/api/workspaces/${activeWorkspace.id()}/settings`,
    {}
  );

  return (
    <div class="w-full h-full flex flex-col">
      <div class="pt-4 pb-8 text-center text-2xl font-semibold text-gray-900">
        Settings
      </div>
      <div class="flex justify-center">
        <div class="flex-1 p-4 max-w-[650px]">
          <Show when={settingsQuery.data()}>
            <AIModelsConfig
              models={settingsQuery.data.models()!}
              refreshConfig={() => settingsQuery.refresh()}
            />
          </Show>
        </div>
      </div>
    </div>
  );
};

const AIModelsConfig = (props: {
  models: Workspace.Model[];
  refreshConfig: () => void;
}) => {
  return (
    <ConfigSection title="AI Models">
      <div class="text-sm space-y-2">
        <For each={props.models}>
          {(model) => {
            return (
              <AIModelConfig {...model} refreshConfig={props.refreshConfig} />
            );
          }}
        </For>
        <div class="pt-6 flex justify-end">
          <button
            type="button"
            class="px-4 py-1.5 max-w-[200px] rounded text-white bg-indigo-600 hover:bg-indigo-500"
          >
            Add custom model
          </button>
        </div>
      </div>
    </ConfigSection>
  );
};

const AIModelConfig = (
  props: Workspace.Model & { refreshConfig: () => void }
) => {
  const { activeWorkspace, refreshWorkspace } = useSharedWorkspaceContext();
  const deleteModel = createMutationQuery<{ id: string }>((input) => {
    return {
      url: `/api/llm/models/delete`,
      request: {
        body: {
          id: input.id,
          workspaceId: activeWorkspace.id(),
        },
      },
    };
  });

  const updateModel = createMutationQuery<{ id: string; metadata: any }>(
    (input) => {
      return {
        url: `/api/llm/models/update`,
        request: {
          body: {
            id: input.id,
            workspaceId: activeWorkspace.id(),
            metadata: input.metadata,
          },
        },
      };
    }
  );

  return (
    <div class="flex">
      <div class="flex-1 py-2 font-medium">{props.name}</div>

      <div class="flex space-x-3 items-center">
        <Show when={props.custom}>
          <HiOutlineTrash
            size={18}
            class="p-0.5 rounded cursor-pointer hover:bg-gray-200"
            onClick={() =>
              deleteModel
                .mutate({
                  id: props.id,
                })
                .then(() => {
                  props.refreshConfig();
                  refreshWorkspace();
                })
            }
          />
        </Show>

        <Show when={props.custom}>
          <label class="group px-4 py-1.5 w-32 flex justify-start items-center text-sm rounded-md cursor-pointer has-[:checked]:border border-gray-100 has-[:checked]:bg-indigo-50 has-[:checked]:border-indigo-200">
            <div class="w-20 hidden group-has-[:checked]:block text-gray-700">
              Enabled
            </div>
            <div class="w-20 group-has-[:checked]:hidden text-gray-500">
              Disabled
            </div>
            <input
              class="ml-4"
              type="checkbox"
              checked={!props.disabled}
              onChange={(e) => {
                updateModel
                  .mutate({
                    id: props.id,
                    metadata: {
                      disabled: !e.target.checked,
                    },
                  })
                  .then(() => {
                    props.refreshConfig();
                    refreshWorkspace();
                  });
              }}
            />
          </label>
        </Show>
      </div>
    </div>
  );
};

const ConfigSection = (props: { title: string; children: any }) => {
  return (
    <div class="space-y-4">
      <div class="text-xl font-semibold text-slate-700 border-b border-gray-300">
        {props.title}
      </div>
      <div class="text-gray-700">{props.children}</div>
    </div>
  );
};

export default WorkspaceSettings;
