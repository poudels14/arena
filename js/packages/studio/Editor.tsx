import { Title } from "@solidjs/meta";
import {
  EditorStateConfig,
  TemplateStoreContext,
  createEditorWithPlugins,
  useEditorContext,
  withEditorState,
  withTemplateStore,
  withWidgetDataLoaders,
  withComponentTree,
  ComponentTreeContext,
  withKeyboardShortcuts,
} from "./editor";
import { Canvas } from "./Canvas";
import { Toolbar } from "./toolbar";
import {
  DragDropProvider,
  DragEndEvent,
  DragOverlay,
  useDragDropContext,
} from "@arena/solid-dnd";
import { Match, Switch, createMemo } from "solid-js";
import { Widget } from "./Widget";
import { Slot } from "./Slot";
import { TEMPLATES } from "./templates";

type EditorProps = EditorStateConfig & {};

const Editor = (props: EditorProps) => {
  const AppEditorProvider = createEditorWithPlugins(
    withEditorState({
      appId: props.appId,
    }),
    withComponentTree(),
    withTemplateStore({
      templates: TEMPLATES,
    }),
    withWidgetDataLoaders({}),
    withKeyboardShortcuts({})
  );

  return (
    <DragDropProvider
      options={{
        collision: {
          distance: 80,
        },
      }}
    >
      <AppEditorProvider>
        <AppEditor />
      </AppEditorProvider>
      <DragOverlay />
    </DragDropProvider>
  );
};

const AppEditor = () => {
  const { state, addWidget, updateWidget, useChildren, isViewOnly } =
    useEditorContext<TemplateStoreContext & ComponentTreeContext>();
  const { attachDragEndHandler } = useDragDropContext();

  const getRootWidget = createMemo(() => useChildren(null)[0]);
  const onDragEnd = async (e: DragEndEvent) => {
    const { templateId, widgetId } = e.draggable.data || {};
    if (e.droppable) {
      const { parentId, afterWidget } = e.droppable!.data;
      const children = useChildren(parentId);
      const afterIndex = children.findIndex((c) => c == afterWidget);
      const before =
        afterIndex + 1 < children.length ? children[afterIndex + 1] : null;
      if (templateId) {
        await addWidget({
          parentId,
          templateId,
          config: {
            layout: {
              position: {
                after: afterWidget,
                before,
              },
            },
          },
        });
        // if widgetId is set, update the widget's position
      } else if (widgetId) {
        updateWidget(widgetId, "config", "layout", "position", {
          after: afterWidget,
          // @ts-expect-error
          before,
        });
      }
    }
  };
  attachDragEndHandler(onDragEnd);
  return (
    <>
      <Title>{state.app.name()}</Title>
      <div
        class="w-full h-full min-w-[900px] no-scrollbar"
        classList={{
          "pb-64": !isViewOnly(),
        }}
      >
        <Canvas>
          <Switch>
            <Match when={getRootWidget()}>
              <Widget widgetId={getRootWidget()} />
            </Match>
            <Match when={true}>
              <Slot parentId={null} />
            </Match>
          </Switch>
        </Canvas>
      </div>
      <Toolbar />
    </>
  );
};

export { Editor };
