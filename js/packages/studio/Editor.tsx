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
import Heading1, {
  metadata as heading1,
} from "@arena/widgets/builtin/Heading1";
import Heading2, {
  metadata as heading2,
} from "@arena/widgets/builtin/Heading2";
import Heading3, {
  metadata as heading3,
} from "@arena/widgets/builtin/Heading3";
import Table, { metadata as tableMetadata } from "@arena/widgets/builtin/table";
import Chart, { metadata as chartMetadata } from "@arena/widgets/builtin/Chart";
import GridLayout, {
  metadata as gridLayoutMetadata,
} from "@arena/widgets/builtin/GridLayout";
import { Canvas } from "./canvas";
import { ComponentTree } from "./ComponentTree";
import { Toolbar } from "./toolbar";
import { DragDropProvider, DragEndEvent, DragOverlay } from "@arena/solid-dnd";
import { Match, Switch } from "solid-js";
import { Widget } from "./Widget";
import { Slot } from "./Slot";

type EditorProps = EditorStateConfig & {};

const Editor = (props: EditorProps) => {
  const AppEditorProvider = createEditorWithPlugins(
    withEditorState({
      appId: props.appId,
    }),
    withComponentTree(),
    withResizer({}),
    withTemplateStore({
      templates: {
        // TODO(sagar): make these lazy load
        [heading1.id]: {
          Component: Heading1,
          metadata: heading1,
        },
        [heading2.id]: {
          Component: Heading2,
          metadata: heading2,
        },
        [heading3.id]: {
          Component: Heading3,
          metadata: heading3,
        },
        [gridLayoutMetadata.id]: {
          Component: GridLayout,
          metadata: gridLayoutMetadata,
        },
        [tableMetadata.id]: {
          Component: Table,
          metadata: tableMetadata,
        },
        [chartMetadata.id]: {
          Component: Chart,
          metadata: chartMetadata,
        },
      },
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

  const onDragEnd = async (e: DragEndEvent) => {
    const templateId = e.draggable.data.templateId;
    const template = useTemplate(templateId);
    if (e.droppable) {
      await addWidget(
        e.droppable!.data.parentId,
        templateId,
        template.metadata
      );
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
                  <Match when={useChildren(null).length > 0}>
                    <Widget widgetId={useChildren(null)[0]} />
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
