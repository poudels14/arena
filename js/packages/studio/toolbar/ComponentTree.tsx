import { For, Show } from "solid-js";
import { InlineIcon } from "@arena/components";
// TODO(sagar): figure out a way to use exports from `@blueprintjs/icons` without
// crashing the browser; browser crashes because it tries to load all exported icons
import ListDetailView from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/list-detail-view";
import { ComponentTreeNode, useEditorContext } from "../editor";

const NodeComponent = (props: {
  node: ComponentTreeNode;
  isWidgetSelected: (id: string) => boolean;
  selectWiget: (id: string) => void;
}) => {
  return (
    <div class="node space-y-[1px]">
      <div class="flex">
        <div
          class="title px-2 py-1 space-x-1 rounded cursor-pointer"
          classList={{
            "bg-accent-12/90 text-accent-1 shadow-lg": props.isWidgetSelected(
              props.node.id!
            ),
            "bg-accent-12/70 text-accent-2": !props.isWidgetSelected(
              props.node.id!
            ),
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
                isWidgetSelected={props.isWidgetSelected}
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
  const { isWidgetSelected, setSelectedWidget } = useEditorContext();
  return (
    <div class="component-tree">
      <Show when={props.node}>
        {/* Note(sagar): this rerenders the entire tree every time tree is changed */}
        <NodeComponent
          node={props.node!}
          isWidgetSelected={isWidgetSelected}
          selectWiget={setSelectedWidget}
        />
      </Show>
    </div>
  );
};

export { ComponentTree };
