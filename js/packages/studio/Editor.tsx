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
import { Canvas } from "./canvas";
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
  const { state, addWidget, useTemplate, getComponentTree, useChildren } =
    useEditorContext<TemplateStoreContext & ComponentTreeContext>();

  const getChildren = createMemo(() => useChildren(null));
  const onDragEnd = async (e: DragEndEvent) => {
    const templateId = e.draggable.data.templateId;
    if (e.droppable) {
      const { parentId, position } = e.droppable!.data;
      await addWidget({ parentId, templateId, position });
    }
  };

  return (
    <DragDropProvider onDragEnd={onDragEnd}>
      <div class="relative min-w-[900px] h-screen">
        <Title>{state.app.name()}</Title>
        <div class="absolute top-8 left-6 z-[10000]">
          <ComponentTree node={getComponentTree()} />
        </div>
        <Toolbar />
        <div class="w-full h-full">
          {/* <div class="fixed bg-red-100 w-full h-8">DO WE NEED APP HEADER BAR?</div> */}
          <div class="h-full">
            <Canvas>
              <div class="p-2">
                <Switch>
                  <Match when={getChildren().length > 0}>
                    <Widget widgetId={getChildren()[0]} />
                  </Match>
                  <Match when={true}>
                    <Slot parentId={null} />
                  </Match>
                </Switch>
              </div>
            </Canvas>
          </div>
        </div>
      </div>
      <DragOverlay />
    </DragDropProvider>
  );
};

export { Editor };
