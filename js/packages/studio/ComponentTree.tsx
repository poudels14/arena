import { For, Show } from "solid-js";
import { InlineIcon } from "@arena/components";
// TODO(sagar): figure out a way to use exports from `@blueprintjs/icons` without
// crashing the browser; browser crashes because it tries to load all exported icons
import ListDetailView from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/list-detail-view";
import { ComponentTreeNode, useEditorContext } from "./editor";

const NodeComponent = (props: {
  node: ComponentTreeNode;
  selectedWidgetIds: string[];
  selectWiget: (id: string) => void;
}) => {
  const isSelected = () => props.selectedWidgetIds.includes(props.node.id!);
  return (
    <div class="node space-y-[1px]">
      <div class="flex">
        <div
          class="title px-2 py-1 space-x-1 rounded cursor-pointer"
          classList={{
            "bg-slate-400 text-gray-800": isSelected(),
            "bg-slate-700 text-white": !isSelected(),
          }}
          onClick={() => props.node.id && props.selectWiget(props.node.id)}
        >
          <InlineIcon size="10px" class="inline-block pt-[2px]">
            <path d={ListDetailView[0]} />
          </InlineIcon>
          <div class="inline-block text-[10px] leading-[10px]">
            {props.node.name}
          </div>
        </div>
      </div>
      <div class="children pl-4">
        <For each={props.node.children || []}>
          {(node) => {
            return (
              <NodeComponent
                node={node}
                selectedWidgetIds={props.selectedWidgetIds}
                selectWiget={props.selectWiget}
              />
            );
          }}
        </For>
      </div>
    </div>
  );
};

const ComponentTree = (props: { node: ComponentTreeNode | null }) => {
  const { getSelectedWidgets, setSelectedWidget } = useEditorContext();
  return (
    <div class="component-tree">
      <Show when={props.node}>
        {/* Note(sagar): this rerenders the entire tree every time tree is changed */}
        <NodeComponent
          node={props.node!}
          selectedWidgetIds={getSelectedWidgets().map((id) => id!)}
          selectWiget={setSelectedWidget}
        />
      </Show>
    </div>
  );
};

export { ComponentTree };
