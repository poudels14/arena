import { Title } from "@solidjs/meta";
import {
  EditorStateConfig,
  TemplateStoreContext,
  createEditorWithPlugins,
  useEditorContext,
  withWidgetProps,
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
import { Match, Switch, createMemo, lazy } from "solid-js";
import { Widget } from "./Widget";
import { Slot } from "./Slot";
import { TEMPLATES } from "./templates";
import { App } from "./types";

type EditorProps = EditorStateConfig & {};

const Editor = (props: EditorProps) => {
  const AppEditorProvider = createEditorWithPlugins(
    withEditorState({
      appId: props.appId,
    }),
    withWidgetProps(),
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
      <div class="w-full h-full min-w-[768px] no-scrollbar">
        <Canvas showGrid={!isViewOnly()}>
          <Switch>
            <Match when={state.app.template()}>
              <AppWithTemplate app={state.app()} />
            </Match>
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

const AppWithTemplate = (props: { app: App }) => {
  const template = props.app.template!;
  const Component = lazy(
    () => import(`/static/templates/apps/${template.id}/${template.version}.js`)
  );
  return <Component />;
};

export { Editor };
