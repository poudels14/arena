import { For } from "solid-js";
import { InlineIcon } from "@arena/components";
// TODO(sagar): figure out a way to use exports from `@blueprintjs/icons` without
// crashing the browser; browser crashes because it tries to load all exported icons
import ListDetailView from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/list-detail-view";
import { Store } from "@arena/solid-store";
import { Node } from "./state";

const NodeComponent = (props: { node: Store<Node> }) => {
  return (
    <div class="node space-y-[1px]">
      <div class="flex">
        <div class="title px-2 py-1 space-x-1 rounded text-white bg-slate-700 cursor-pointer">
          <InlineIcon size="10px" class="inline-block pt-[2px]">
            <path d={ListDetailView[0]} />
          </InlineIcon>
          <div class="inline-block text-[10px] leading-[10px]">
            {props.node.title}
          </div>
        </div>
      </div>
      <div class="children pl-4">
        <For each={props.node.children || []}>
          {(node) => {
            return <NodeComponent node={node} />;
          }}
        </For>
      </div>
    </div>
  );
};

const ComponentTree = (props: { node: Node }) => {
  return (
    <div class="component-tree">
      {/* Note(sagar): this rerenders the entire tree every time tree is changed */}
      <NodeComponent node={props.node} />
    </div>
  );
};

export { ComponentTree };
