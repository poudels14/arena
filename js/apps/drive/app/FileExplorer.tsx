import {
  For,
  Match,
  Show,
  Switch,
  createMemo,
  createComputed,
  createSignal,
  createSelector,
} from "solid-js";
import { createQuery } from "@portal/solid-query";
import { useLocation, useMatcher, useNavigate } from "@portal/solid-router";
import { useSharedWorkspaceContext } from "@portal/workspace-sdk";
import { createDroppable } from "@portal/solid-dnd";

import { Directory, File } from "./components/File";
import { FileProperties } from "./FileProperties";
import { Header } from "./Header";

export type Directory = {
  id: string;
  name: string;
  parentId: string | null;
  type?: string;
  isDirectory: boolean;
  breadcrumbs: Pick<Directory, "id" | "name">[];
  children: Directory[];
  appId?: string;
};

const FileExplorer = () => {
  const { getCurrentApp, setChatContext } = useSharedWorkspaceContext();
  const [getSelectedFile, setSelectedFile] = createSignal<null | Directory>(
    null
  );
  const directoryId = useMatcher(() => `/:appId/explore/:id`);
  const currentDirectoryId = createMemo(() => {
    return directoryId()?.params?.id || null;
  });

  const navigate = useNavigate();
  const location = useLocation();
  const goToDirectory = (id: string, appId?: string) => {
    let query = "";
    if (appId) {
      query = `?app=${appId}`;
    }
    navigate(`/${getCurrentApp()!.id}/explore/` + id + query);
  };
  const filesQuery = createQuery<Directory>(() => {
    const id = currentDirectoryId() ?? "";
    const appId = location.searchParams.find((p) => p[0] == "app");
    let query = "";
    if (appId) {
      query = `?app=${appId[1]}`;
    }
    return `/api/fs/directory/${id}${query}`;
  }, {});

  const isFileSelected = createSelector(() => getSelectedFile()?.id);

  createComputed(() => {
    // reset selection when directory changes
    void currentDirectoryId();
    setSelectedFile(null);
    filesQuery.refresh();
  });

  const chatContextBreadcrums = createMemo<any[]>((prev) => {
    const data = filesQuery.data();
    if (data) {
      return data.breadcrumbs.map((crumb) => {
        return {
          id: crumb.id,
          title: crumb.name,
        };
      });
    }
    // Note(sagar): return prev here such that until the new dir data is loaded,
    // the previous breadcrumb is intact. This will prevent flickering since
    // `setChatContext` is reactive and clears previous context if called from
    // reactive context
    return prev;
  }, []);

  createComputed(() => {
    const breadcrumbs = [...chatContextBreadcrums()];
    const selection = getSelectedFile();
    if (selection) {
      breadcrumbs.push({
        id: selection.id,
        title: selection.name,
      });
    }

    const selction = breadcrumbs[breadcrumbs.length - 1];
    setChatContext({
      app: getCurrentApp()!,
      selection: selction
        ? {
            id: selction.id,
            type: selction.isDirectory ? "directory" : "file",
          }
        : undefined,
      breadcrumbs,
    });
  });

  const droppable = createDroppable(`drive-file-explorer`, {});

  return (
    <div class="file-explorer flex h-full">
      <div class="flex flex-col flex-1">
        <Header
          currentDir={currentDirectoryId()}
          selected={getSelectedFile()}
          breadcrumbs={chatContextBreadcrums()}
          onUpload={() => {
            filesQuery.refresh();
          }}
          onNewDirectory={() => filesQuery.refresh()}
          onClickBreadcrumb={(id) => goToDirectory(id)}
        />
        <div
          class="files flex-1 px-8 py-4 border-4"
          ref={droppable.ref}
          classList={{
            "border-indigo-300": droppable.isActiveDroppable,
            "border-transparent": !droppable.isActiveDroppable,
          }}
        >
          <div class="flex flex-wrap gap-6 text-xs">
            <Show when={filesQuery.data.children()}>
              <Show when={currentDirectoryId() != null}>
                <Directory
                  id={filesQuery.data.parentId() || "root"}
                  name={".."}
                  selected={false}
                  onClick={() => {
                    setSelectedFile(null);
                  }}
                  onDblClick={() => {
                    goToDirectory(filesQuery.data.parentId() ?? "");
                  }}
                />
              </Show>
              <Show when={currentDirectoryId() == null}>
                <SharedWithMe />
              </Show>
              <For each={filesQuery.data.children()}>
                {(file) => {
                  return (
                    <Switch>
                      <Match when={file.isDirectory}>
                        <Directory
                          id={file.id}
                          name={file.name}
                          selected={isFileSelected(file.id)}
                          onClick={() => {
                            setSelectedFile(file);
                          }}
                          onDblClick={() => {
                            goToDirectory(file.id, file.appId);
                          }}
                        />
                      </Match>
                      <Match when={!file.isDirectory}>
                        <File
                          id={file.id}
                          name={file.name}
                          selected={isFileSelected(file.id)}
                          type={file.type!}
                          onClick={() => {
                            setSelectedFile(file);
                          }}
                          onDblClick={() => {
                            goToDirectory(file.id, file.appId);
                          }}
                        />
                      </Match>
                    </Switch>
                  );
                }}
              </For>
            </Show>
            <Show when={filesQuery.status() == 404}>
              <div class="flex-1">
                <div class="text-center font-semibold text-lg">Not found</div>
              </div>
            </Show>
          </div>
        </div>
      </div>
      <div class="sm:w-0 lg:w-96 h-full border-l border-gray-100 bg-gray-50">
        <FileProperties selected={getSelectedFile()} />
      </div>
    </div>
  );
};

const SharedWithMe = () => {
  const { getCurrentApp } = useSharedWorkspaceContext();
  const navigate = useNavigate();
  const goToDirectory = (id: string) =>
    navigate(`${getCurrentApp()!.id}/explore/` + id);
  return (
    <Directory
      id={"shared"}
      name={"Shared with me"}
      selected={false}
      onClick={() => {}}
      onDblClick={() => {
        goToDirectory("shared");
      }}
    />
  );
};

export { FileExplorer };
