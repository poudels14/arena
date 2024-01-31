import { Show, createContext, useContext } from "solid-js";
import { createQuery } from "@portal/solid-query";
import { Store } from "@portal/solid-store";

type WorkspaceContext = {
  activeWorkspace: Store<Workspace>;
};

type Workspace = {
  id: string;
  name: string;
  apps: App[];
};

type App = {
  id: string;
  name: string;
  slug: string;
};

const WorkspaceContext = createContext<WorkspaceContext>();
const useWorkspaceContext = () => useContext(WorkspaceContext)!;

type WorkspaceContextProviderProps = {
  urlPrefix?: string;
  children: any;
};

const WorkspaceContextProvider = (props: WorkspaceContextProviderProps) => {
  const workspaces = createQuery<Workspace[]>(() => "/api/workspaces", {});
  const workspaceQuery = createQuery<Workspace>(() => {
    if (!workspaces.data()) {
      return null;
    }
    // TODO: use active workspace
    const workspace = workspaces.data()![0];
    return `/api/workspaces/${workspace.id}`;
  }, {});

  const value = {
    get activeWorkspace() {
      return workspaceQuery?.data!;
    },
  };

  return (
    <Show when={value.activeWorkspace?.()}>
      {/* @ts-ignore */}
      <WorkspaceContext.Provider value={value}>
        {props.children}
      </WorkspaceContext.Provider>
    </Show>
  );
};

export { useWorkspaceContext, WorkspaceContextProvider };
