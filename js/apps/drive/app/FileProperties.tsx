import { Show } from "solid-js";
import { Directory } from "./FileExplorer";
import { useSharedWorkspaceContext } from "@portal/workspace-sdk";

type FilePropertiesProps = {
  selected: Directory | null;
};

const FileProperties = (props: FilePropertiesProps) => {
  const { shareEntities } = useSharedWorkspaceContext();
  return (
    <Show when={props.selected}>
      <div>
        <div class="flex py-2 px-2 font-bold text-center text-gray-700 bg-gray-100 shadow-sm">
          <div class="flex-1">{props.selected?.name}</div>
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
        </div>
      </div>
    </Show>
  );
};

export { FileProperties };
