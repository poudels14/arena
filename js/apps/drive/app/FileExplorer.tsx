import { For, Match, Show, Switch, createMemo, createComputed } from "solid-js";
import { createQuery } from "@portal/solid-query";
import { useMatcher, useNavigate } from "@portal/solid-router";
import { useSharedWorkspaceContext } from "@portal/workspace-sdk";
import { Directory, File } from "./components/File";
import { Uploader } from "./Uploader";

type Directory = {
  id: string;
  name: string;
  parentId: string | null;
  type?: string;
  isDirectory: boolean;
  breadcrumbs: Pick<Directory, "id" | "name">[];
  children: Directory[];
};

const FileExplorer = () => {
  const { getCurrentApp, setChatContext } = useSharedWorkspaceContext();
  const directoryId = useMatcher("/explore/:id");
  const currentDirectoryId = createMemo(() => {
    return directoryId()?.params?.id || null;
  });

  const navigate = useNavigate();
  const filesQuery = createQuery<Directory>(() => {
    return `/api/fs/directory?id=${currentDirectoryId()}`;
  }, {});

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
    setChatContext({
      app: getCurrentApp(),
      breadcrumbs: chatContextBreadcrums(),
    });
  });

  return (
    <div class="px-8 py-4">
      <Uploader
        parentId={currentDirectoryId()}
        onUpload={() => {
          filesQuery.refresh();
        }}
      />
      <div class="flex gap-6 text-xs">
        <Show when={filesQuery.data.children()}>
          <Show when={currentDirectoryId() != null}>
            <Directory
              id={filesQuery.data.parentId() || "root"}
              name={".."}
              onDblClick={() => {
                navigate(`/explore/` + (filesQuery.data.parentId() ?? ""));
              }}
            />
          </Show>
          <For each={filesQuery.data.children()}>
            {(file) => {
              return (
                <Switch>
                  <Match when={file.isDirectory}>
                    <Directory
                      id={file.id}
                      name={file.name}
                      onDblClick={() => {
                        navigate(`/explore/` + file.id);
                      }}
                    />
                  </Match>
                  <Match when={!file.isDirectory}>
                    <File
                      id={file.id}
                      name={file.name}
                      type={file.type!}
                      onDblClick={() => {
                        navigate(`/explore/` + file.id);
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
  );
};

export { FileExplorer };
