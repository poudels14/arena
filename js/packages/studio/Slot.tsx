import { For, Show, createMemo } from "solid-js";
import { useDragDropContext, createDroppable } from "@arena/solid-dnd";
import { Draggable } from "@arena/solid-dnd/draggable";
import { ComponentTreeContext, useEditorContext } from "./editor";
import { Widget } from "./Widget";
import { Template } from "@arena/widgets";

type Slot = Template.Props<any>["Editor"]["Slot"];

const Slot: Slot = (props) => {
  const droppable = createDroppable("slot-" + props.parentId, {
    parentId: props.parentId,
  });

  const { state } = useDragDropContext();
  const activeDraggable = createMemo(() => state.active.draggable());
  const { useChildren } = useEditorContext<ComponentTreeContext>();

  return (
    <div ref={droppable.ref} class="widget-slot">
      <For each={useChildren(props.parentId)}>
        {(child) => <Widget widgetId={child} />}
      </For>
      <Show when={droppable.isActiveDroppable}>
        <PreviewTemplate draggable={activeDraggable()!} />
      </Show>
    </div>
  );
};

const PreviewTemplate = (props: { draggable: Draggable }) => {
  return <div class="h-1 w-full bg-blue-400"></div>;
};

export { Slot };
