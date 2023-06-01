import { For, JSX, Match, Show, Switch, createMemo } from "solid-js";
import { useDragDropContext, createDroppable } from "@arena/solid-dnd";
import { Draggable } from "@arena/solid-dnd/draggable";
import { ComponentTreeContext, useEditorContext } from "./editor";
import { Widget } from "./Widget";
import { Template } from "@arena/widgets";

type Slot = Template.Props<any>["Editor"]["Slot"];

const Slot: Slot = (props) => {
  const { state } = useDragDropContext();
  const activeDraggable = createMemo(() => state.active.draggable());
  const { useChildren } = useEditorContext<ComponentTreeContext>();
  const children = createMemo(() => useChildren(props.parentId));
  return (
    <Switch>
      <Match when={children().length > 0}>
        <Droppable
          parentId={props.parentId}
          afterWidget={null}
          activeDraggable={activeDraggable()}
        />
        <For each={children()}>
          {(child) => {
            return (
              <>
                <Widget widgetId={child} />
                <Droppable
                  parentId={props.parentId}
                  afterWidget={child}
                  activeDraggable={activeDraggable()}
                />
              </>
            );
          }}
        </For>
      </Match>
      <Match when={true}>
        <Placeholder parentId={props.parentId}>{props.children}</Placeholder>
      </Match>
    </Switch>
  );
};

const Droppable = (props: {
  parentId: string | null;
  afterWidget: string | null;
  activeDraggable: Draggable | null;
}) => {
  const droppable = createDroppable(
    `slot-${props.parentId}-${props.afterWidget}`,
    {
      parentId: props.parentId,
      afterWidget: props.afterWidget,
    }
  );
  return (
    <div ref={droppable.ref} class="slot">
      <Show when={droppable.isActiveDroppable}>
        <PreviewTemplate draggable={props.activeDraggable!} />
      </Show>
    </div>
  );
};

const Placeholder = (props: {
  parentId: string | null;
  children?: JSX.Element;
}) => {
  const droppable = createDroppable(`slot-${props.parentId}-null`, {
    parentId: props.parentId,
    afterWidget: null,
  });

  return (
    <div
      ref={droppable.ref}
      class="placeholder px-5 py-5 text-center text-accent-10 border-2 border-gray-500 border-dashed"
      classList={{
        "bg-white": !droppable.isActiveDroppable,
        "shadow-inner shadow-brand-11 bg-brand-2": droppable.isActiveDroppable,
      }}
    >
      {props.children || "Drop widget templates here"}
    </div>
  );
};

const PreviewTemplate = (props: { draggable: Draggable }) => {
  return <div class="preview bg-blue-400"></div>;
};

export { Slot };
