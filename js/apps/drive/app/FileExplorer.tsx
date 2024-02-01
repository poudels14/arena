import { createQuery } from "@portal/solid-query";
import { useMatcher, useNavigate } from "@portal/solid-router";
import { Directory, File } from "./components/File";
import { For, Match, Show, Switch, createMemo } from "solid-js";
import { Uploader } from "./Uploader";

type Directory = {
  id: string;
  name: string;
  parentId: string | null;
  type?: string;
  isDirectory: boolean;
  children: Directory[];
};

const FileExplorer = () => {
  const directoryId = useMatcher("/files/:id");
  const currentDirectoryId = createMemo(() => {
    return directoryId()?.params?.id || null;
  });

  const navigate = useNavigate();
  const filesQuery = createQuery<Directory>(() => {
    return `/api/fs/directory?id=${currentDirectoryId()}`;
  }, {});

  return (
    <div class="p-2">
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
                navigate(`/files/` + (filesQuery.data.parentId() ?? ""));
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
                        navigate(`/files/` + file.id);
                      }}
                    />
                  </Match>
                  <Match when={!file.isDirectory}>
                    <File
                      id={file.id}
                      name={file.name}
                      type={file.type!}
                      onDblClick={() => {
                        navigate(`/files/` + file.id);
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
