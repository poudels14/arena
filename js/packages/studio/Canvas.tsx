import { Show } from "solid-js";
import { createStore } from "@arena/solid-store";
import { useEditorContext } from "./editor";

const GridLines = (props: { width: number; height: number; scale: number }) => {
  return (
    <div
      classList={{
        [`bg-[length:${props.width * props.scale}px_${
          props.height * props.scale
        }px] `]: true,
      }}
      class="absolute w-full h-full bg-[linear-gradient(to_right,transparent,transparent,99%,rgba(51,65,85,0.2)),linear-gradient(to_top,transparent,transparent,99%,rgba(51,65,85,0.2))]"
    ></div>
  );
};

const Canvas = (props: { showGrid: boolean; children: any }) => {
  const { isViewOnly } = useEditorContext<any>();
  const [state, setState] = createStore({
    scale: 1,
  });

  return (
    <div class="arena-editor relative w-full h-full">
      <Show when={props.showGrid}>
        <GridLines width={40} height={30} scale={state.scale()} />
      </Show>
      <div
        class="arena-canvas-container relative w-full h-full overflow-x-auto overflow-y-auto"
        classList={{
          "pb-64 no-scrollbar": !isViewOnly(),
        }}
      >
        <div
          class="arena-canvas relative"
          style={{
            transform: `scale(${state.scale()})`,
          }}
        >
          {props.children}
        </div>
      </div>
    </div>
  );
};

export { Canvas };
