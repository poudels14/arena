import { createStore } from "@arena/solid-store";

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

const Canvas = (props: { children: any }) => {
  const [state, setState] = createStore({
    scale: 1,
  });

  return (
    <div class="arena-editor relative w-full h-full">
      <GridLines width={40} height={30} scale={state.scale()} />
      <div class="arena-canvas-container relative w-full h-full overflow-x-auto overflow-y-auto">
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
