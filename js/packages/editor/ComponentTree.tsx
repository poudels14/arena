import { InlineIcon } from "@arena/components";
// TODO(sagar): figure out a way to use exports from `@blueprintjs/icons` without
// crashing the browser; browser crashes because it tries to load all exported icons
import ListDetailView from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/list-detail-view";
import { StoreValue } from "@arena/solid-store";
import { Node } from "./state";

const NodeComponent = (props: { node: StoreValue<Node> }) => {
  return (
    <div class="flex flex-row px-2 py-1 space-x-1 cursor-pointer">
      <InlineIcon size="14px">
        <path d={ListDetailView[0]} />
      </InlineIcon>
      <div class="text-xs">{props.node.title}</div>
    </div>
  );
};

const ComponentTree = (props: { node: StoreValue<Node> }) => {
  return (
    <div class="rounded text-white bg-slate-700">
      <NodeComponent node={props.node} />
    </div>
  );
};

export { ComponentTree };
