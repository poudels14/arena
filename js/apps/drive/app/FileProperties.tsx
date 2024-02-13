import { Show } from "solid-js";
import { Directory } from "./FileExplorer";

type FilePropertiesProps = {
  selected: Directory | null;
};

const FileProperties = (props: FilePropertiesProps) => {
  return (
    <Show when={props.selected}>
      <div>
        <div class="py-2 px-2 font-bold text-center text-gray-700 bg-gray-100 shadow-sm">
          {props.selected?.name}
        </div>
      </div>
    </Show>
  );
};

export { FileProperties };
