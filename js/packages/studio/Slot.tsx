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
    <div class="widget-slot">
      <Switch>
        <Match when={children().length > 0}>
          <Droppable
            parentId={props.parentId}
            position={{
              after: null,
              before: children()[0] || null,
            }}
            activeDraggable={activeDraggable()}
          />
          <For each={children()}>
            {(child, index) => {
              return (
                <>
                  <Widget widgetId={child} />
                  <Droppable
                    parentId={props.parentId}
                    position={{
                      after: child,
                      before:
                        index() + 1 < children().length
                          ? children()[index() + 1]
                          : null,
                    }}
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
    </div>
  );
};

const Droppable = (props: {
  parentId: string | null;
  position: { after: string | null; before: string | null };
  activeDraggable: Draggable | null;
}) => {
  const droppable = createDroppable(
    `slot-${props.parentId}-${props.position.after}`,
    {
      parentId: props.parentId,
      position: props.position,
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
    position: {
      after: null,
      before: null,
    },
  });

  return (
    <div
      ref={droppable.ref}
      class="placeholder px-5 py-5 text-center text-gray-600 border-2 border-gray-500 border-dashed"
      classList={{
        "bg-white": !droppable.isActiveDroppable,
        "shadow-inner shadow-slate-400 bg-slate-100":
          droppable.isActiveDroppable,
      }}
    >
      {props.children || "Drop widget templates here"}
    </div>
  );
};

const PreviewTemplate = (props: { draggable: Draggable }) => {
  return <div class="h-1 w-full bg-blue-400"></div>;
};

export { Slot };
