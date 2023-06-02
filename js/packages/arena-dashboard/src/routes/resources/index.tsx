import { createStore } from "@arena/solid-store";
import AddResource from "./AddResource";
import { Show } from "solid-js";
import { ResourcesTable } from "./ResourcesTable";

const Resources = () => {
  const [state, setState] = createStore({
    addResourceDialogOpen: false,
    resourcesRefreshedAt: new Date(),
  });

  return (
    <div class="w-full h-full overflow-y-auto">
      <div class="mt-10 px-4 pb-14 sm:px-6 lg:px-8">
        <div class="sm:flex sm:items-center">
          <div class="sm:flex-auto">
            <h1 class="text-xl font-semibold leading-6 text-accent-12/90">
              Resources
            </h1>
            <p class="mt-2 text-sm font-light text-accent-11">
              A list of all the resources in your account. All Postgres db,
              config, environment variables, etc will be shown here.
            </p>
          </div>
          <div class="mt-4 sm:ml-10 sm:mt-0 sm:flex-none">
            <button
              class="px-3 py-1.5 rounded text-xs border border-accent-11/60 hover:bg-accent-2 select-none"
              onClick={() => setState("addResourceDialogOpen", true)}
            >
              Add new resource
            </button>
          </div>
        </div>
        <div class="mt-8 flow-root">
          <ResourcesTable refresh={state.resourcesRefreshedAt()} />
        </div>
      </div>

      <Show when={state.addResourceDialogOpen()}>
        <AddResource
          closeDialog={() => {
            setState("addResourceDialogOpen", false);
            setState("resourcesRefreshedAt", new Date());
          }}
        />
      </Show>
    </div>
  );
};

export { Resources };
