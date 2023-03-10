import { Title } from "@solidjs/meta";
import { Canvas } from "./canvas";
import { ComponentTree } from "./ComponentTree";
import { App, EditorState } from "./state";
import { Toolbar } from "./toolbar";

const Editor = (props: { app: App }) => {
  const state = new EditorState();
  const app = state.getApp();

  return (
    <div class="relative min-w-[900px] h-screen">
      <Title>{app.name}</Title>
      <div class="absolute top-8 left-6 z-[10000]">
        <ComponentTree node={state.getComponentTree()} />
      </div>
      <Toolbar />
      <div class="w-full h-full">
        <Canvas />
      </div>
    </div>
  );
};

export { Editor };
