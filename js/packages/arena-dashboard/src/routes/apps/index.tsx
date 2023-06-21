import { Show, For, createResource } from "solid-js";
import { Title } from "@arena/core/solid";
import { A } from "@solidjs/router";
import { useDashboardContext } from "~/context";
import { createStore } from "@arena/solid-store";
import CreateAppDialog from "./CreateAppDialog";

const App = (props: {
  id: string;
  name: string;
  description?: string;
  config: any;
}) => {
  const thumbnailClass = () => props.config?.ui?.thumbnail?.class;
  return (
    <A
      href={"/apps/" + props.id}
      class="w-48 h-24 lg:w-64 lg:h-36 relative group bg-brand-2 rounded-lg bg-gradient-to-br cursor-pointer"
      classList={{
        [thumbnailClass()]: Boolean(thumbnailClass()),
      }}
    >
      <div class="absolute bottom-0 px-4 py-2">
        <div class="font-medium text-accent-12/80 group-hover:text-brand-12">
          {props.name}
        </div>
      </div>
    </A>
  );
};

const Apps = () => {
  const { client, workspace } = useDashboardContext();
  const [state, setState] = createStore({
    createAppDialogOpen: false,
  });

  const [apps, { refetch }] = createResource(() => {
    return client.apps.list.query({
      workspaceId: workspace.id,
    });
  });

  return (
    <Show when={apps()}>
      <Title>Apps</Title>
      <div class="w-full h-full overflow-auto">
        <div class="w-full px-10 py-10 flex justify-end">
          <button
            class="px-3 py-1.5 rounded text-xs border border-accent-11/60 hover:bg-accent-2 select-none"
            onClick={() => setState("createAppDialogOpen", true)}
          >
            Create new app
          </button>
        </div>
        <div class="flex flex-row flex-wrap justify-items-start p-10 gap-8">
          <Show when={apps()?.length == 0}>
            <div class="w-full py-10 text-sm text-center text-accent-9">
              No apps created yet
            </div>
          </Show>
          <For each={apps()}>
            {(app) => {
              return <App {...app} />;
            }}
          </For>
        </div>
      </div>
      <Show when={state.createAppDialogOpen()}>
        <CreateAppDialog
          closeDialog={() => {
            setState("createAppDialogOpen", false);
            refetch();
          }}
        />
      </Show>
    </Show>
  );
};

export default Apps;
