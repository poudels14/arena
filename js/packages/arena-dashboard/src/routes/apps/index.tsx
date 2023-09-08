import { Show, For, createResource, createEffect, onCleanup } from "solid-js";
import { Title } from "@arena/core/solid";
import { Store, StoreSetter } from "@arena/solid-store";
import {
  Routes as SolidRoutes,
  Route,
  useNavigate,
  useParams,
} from "@solidjs/router";
import { createStore } from "@arena/solid-store";
import { useDashboardContext } from "~/context";
import CreateAppDialog from "./CreateAppDialog";
import AppThumbnail from "./AppThumbnail";

type PageState = {
  createAppDialogOpen: boolean;
};

const Apps = () => {
  const [state, setState] = createStore<PageState>({
    createAppDialogOpen: false,
  });

  return (
    <SolidRoutes>
      <Route
        path="/:id?"
        component={() => {
          const params = useParams();
          createEffect(() => {
            if (params.id == "new") {
              setState("createAppDialogOpen", true);
            }
            onCleanup(() => {
              setState("createAppDialogOpen", false);
            });
          });
          return <Home state={state} setState={setState} />;
        }}
      />
    </SolidRoutes>
  );
};

const Home = (props: {
  state: Store<PageState>;
  setState: StoreSetter<PageState>;
}) => {
  const navigate = useNavigate();
  const { client, workspace } = useDashboardContext();

  const [apps, { refetch }] = createResource(() => {
    return client.apps.list.query({
      workspaceId: workspace.id,
    });
  });

  const deleteApp = async (id: string) => {
    await client.apps.archive.mutate({
      workspaceId: workspace.id,
      id,
    });
    refetch();
  };

  return (
    <Show when={apps()}>
      <Title>Apps</Title>
      <div class="w-full h-full overflow-auto">
        <div class="w-full px-10 py-10 flex justify-end">
          <button
            class="px-3 py-1.5 rounded text-xs border border-accent-11/60 hover:bg-accent-2 select-none"
            onClick={() => {
              navigate("/apps/new");
            }}
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
              return <AppThumbnail {...app} delete={() => deleteApp(app.id)} />;
            }}
          </For>
        </div>
      </div>
      <Show when={props.state.createAppDialogOpen()}>
        <CreateAppDialog
          closeDialog={() => {
            navigate("/apps");
            refetch();
          }}
        />
      </Show>
    </Show>
  );
};

export default Apps;
