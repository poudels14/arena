import { Title } from "@solidjs/meta";
import {
  Widget,
  createEditorWithPlugins,
  useEditorContext,
  withEditorState,
  withResizer,
  withTemplateStore,
  withWidgetDataLoaders,
} from "@arena/appkit/editor";
import Heading1, { metadata as heading1 } from "@arena/widgets/core/Heading1";
import Heading2, { metadata as heading2 } from "@arena/widgets/core/Heading2";
import Heading3, { metadata as heading3 } from "@arena/widgets/core/Heading3";
import Layout, { metadata as layoutMetadata } from "@arena/widgets/core/Layout";
import { Canvas } from "./canvas";
import { ComponentTree } from "./ComponentTree";
import { EditorState } from "./state";
import { Toolbar } from "./toolbar";
import { AppContextProvider } from "@arena/appkit";
import type { App } from "./index";

type EditorProps = {
  appId: string;
  fetchApp: (appId: string) => Promise<App>;
  // updateApp
  // updateWidget
};

const Editor = (props: EditorProps) => {
  const AppEditorProvider = createEditorWithPlugins(
    withEditorState({
      appId: props.appId,
      fetchApp: props.fetchApp,
    }),
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
        [layoutMetadata.id]: {
          Component: Layout,
          metadata: layoutMetadata,
        },
      },
    }),
    withWidgetDataLoaders({})
  );

  return (
    <AppContextProvider>
      <AppEditorProvider>
        <AppEditor />
      </AppEditorProvider>
    </AppContextProvider>
  );
};

const AppEditor = () => {
  const { state } = useEditorContext();
  const state2 = new EditorState();

  return (
    <div class="relative min-w-[900px] h-screen">
      <Title>{state.app.name()}</Title>
      <div class="absolute top-8 left-6 z-[10000]">
        <ComponentTree node={state2.getComponentTree()} />
      </div>
      <Toolbar />
      <div class="w-full h-full">
        {/* <div class="fixed bg-red-100 w-full h-8">DO WE NEED APP HEADER BAR?</div> */}
        <div class="h-full">
          <Canvas>
            <div class="p-2">
              <Widget widgetId={state.app.widgetId()} />
            </div>
          </Canvas>
        </div>
      </div>
    </div>
  );
};

export { Editor };
