import { Show, createSignal } from "solid-js";
import { useSharedWorkspaceContext } from "@portal/workspace-sdk";
import Dialog from "@portal/solid-ui/Dialog";
import { HiOutlineTrash } from "solid-icons/hi";
import { Directory } from "./FileExplorer";
import { createMutationQuery } from "@portal/solid-query";

type FilePropertiesProps = {
  selected: Directory | null;
  refreshDirectory: () => void;
};

const FileProperties = (props: FilePropertiesProps) => {
  const { shareEntities } = useSharedWorkspaceContext();
  const [isDeleteDialogVisible, setDeleteDialogVisibility] =
    createSignal(false);
  return (
    <Show when={props.selected}>
      <div>
        <div class="flex py-2 px-2 font-bold text-center text-gray-700 bg-gray-100 shadow-sm">
          <div class="flex-1">{props.selected?.name}</div>
          <div class="flex align-middle items-center space-x-1">
            <div
              class="px-1 py-0.5 text-xs rounded-sm cursor-pointer hover:bg-gray-200"
              onClick={() => {
                shareEntities({
                  title: "Share " + props.selected?.name,
                  entities: [{ id: props.selected?.id! }],
                  aclOptions: [
                    {
                      id: "read-only",
                      name: "Read Only",
                      description: "",
                    },
                    {
                      id: "full-access",
                      name: "Full access",
                      description: "",
                    },
                  ],
                });
              }}
            >
              Share
            </div>
            <div>
              <HiOutlineTrash
                size={18}
                class="p-0.5 rounded cursor-pointer hover:bg-gray-200"
                onClick={() => setDeleteDialogVisibility(true)}
              />
            </div>
          </div>
        </div>
        <Show when={isDeleteDialogVisible()}>
          <DeleteDialog
            id={props.selected?.id!}
            name={props.selected?.name!}
            isDirectory={props.selected?.isDirectory!}
            refreshDirectory={props.refreshDirectory}
            toggleVisibility={setDeleteDialogVisibility}
          />
        </Show>
      </div>
    </Show>
  );
};

const DeleteDialog = (props: {
  id: string;
  name: string;
  isDirectory: boolean;
  refreshDirectory: () => void;
  toggleVisibility: (show: boolean) => void;
}) => {
  const deleteQuery = createMutationQuery<string>((input) => {
    return {
      url: "/api/fs/files/delete",
      request: {
        body: {
          id: input,
        },
      },
    };
  });
  return (
    <Dialog
      title={() => (
        <div class="title px-8 pt-8 w-full font-medium text-base text-left text-gray-700 border-gray-100">
          Are you sure you want to delete <b>{props.name}</b>
        </div>
      )}
      open={true}
      onOpenChange={props.toggleVisibility}
    >
      <div class="flex px-8 py-4 w-[550px] text-xs justify-end space-x-4">
        <div
          class="px-4 py-1.5 rounded cursor-pointer text-white bg-indigo-500 hover:bg-indigo-600"
          onClick={() =>
            deleteQuery.mutate(props.id).then(() => {
              props.refreshDirectory();
              props.toggleVisibility(false);
            })
          }
        >
          Confirm
        </div>
        <div
          class="px-4 py-1.5 cursor-pointer rounded bg-gray-200"
          onClick={() => props.toggleVisibility(false)}
        >
          Cancel
        </div>
      </div>
    </Dialog>
  );
};

export { FileProperties };
