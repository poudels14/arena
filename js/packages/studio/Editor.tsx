import { Title } from "@solidjs/meta";
import {
  EditorStateConfig,
  TemplateStoreContext,
  createEditorWithPlugins,
  useEditorContext,
  withEditorState,
  withResizer,
  withTemplateStore,
  withWidgetDataLoaders,
  withComponentTree,
  ComponentTreeContext,
} from "./editor";
import { Canvas } from "./Canvas";
import { ComponentTree } from "./ComponentTree";
import { Toolbar } from "./toolbar";
import { DragDropProvider, DragEndEvent, DragOverlay } from "@arena/solid-dnd";
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
    withResizer({}),
    withTemplateStore({
      templates: TEMPLATES,
    }),
    withWidgetDataLoaders({})
  );

  return (
    <AppEditorProvider>
      <AppEditor />
    </AppEditorProvider>
  );
};

const AppEditor = () => {
  const { state, addWidget, getComponentTree, useChildren } = useEditorContext<
    TemplateStoreContext & ComponentTreeContext
  >();

  const getChildren = createMemo(() => useChildren(null));
  const onDragEnd = async (e: DragEndEvent) => {
    const templateId = e.draggable.data.templateId;
    if (e.droppable) {
      const { parentId, afterWidget } = e.droppable!.data;
      const children = useChildren(parentId);
      const afterIndex = children.findIndex((c) => c == afterWidget);
      await addWidget({
        parentId,
        templateId,
        position: {
          after: afterWidget,
          before:
            afterIndex + 1 < children.length ? children[afterIndex + 1] : null,
        },
      });
    }
  };

  return (
    <DragDropProvider
      onDragEnd={onDragEnd}
      options={{
        collision: {
          distance: 80,
        },
      }}
    >
      <Title>{state.app.name()}</Title>
      {/* <div class="fixed bg-slate-0 bg-gradient-to-b from-slate-600 to-slate-700 opacity-100 z-[10000] text-white w-full h-9 shadow-lg">
        <Header />
      </div> */}
      <div class="fixed top-12 left-6 z-[10000]">
        <ComponentTree node={getComponentTree()} />
      </div>
      <div class="absolute px-[2px] w-[calc(100%-4px)] min-w-[900px] h-screen">
        <div class="w-full h-full">
          <Canvas>
            <Switch>
              <Match when={getChildren().length > 0}>
                <Widget widgetId={getChildren()[0]} />
              </Match>
              <Match when={true}>
                <Slot parentId={null} />
              </Match>
            </Switch>
          </Canvas>
        </div>
        <Toolbar />
      </div>
      <DragOverlay />
    </DragDropProvider>
  );
};

export { Editor };
