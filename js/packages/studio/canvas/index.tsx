// import { DragDropProvider, createDraggable } from "@arena/solid-dnd";
import { createStore } from "@arena/solid-store";
// import { SampleApp } from "./SampleApp";
// import { SampleApp } from "./SampleApp2";
// import { SampleApp } from "./SampleAppDynamic";

const GridLines = (props: { width: number; height: number; scale: number }) => {
  return (
    <div
      classList={{
        [`bg-[length:${props.width * props.scale}px_${
          props.height * props.scale
        }px] `]: true,
      }}
      class="absolute w-full h-full border-4 border-[rgba(51,65,85,0.2)] bg-[linear-gradient(to_right,transparent,transparent,99%,rgba(51,65,85,0.2)),linear-gradient(to_top,transparent,transparent,99%,rgba(51,65,85,0.2))]"
    ></div>
  );
};

// const Draggable = (props: any) => {
//   const draggable = createDraggable(props.id);
//   return (
//     <div
//       use:draggable
//       class="draggable absolute left-[800px]  w-52 h-20 bg-green-400"
//       classList={{ "opacity-50": draggable.isActiveDraggable }}
//     >
//       <div>Draggable {props.id}</div>
//       <div>{props.children}</div>
//     </div>
//   );
// };

// export const DragMoveExample = () => {
//   return (
//     <DragDropProvider>
//       <div class="min-h-20 w-full h-full relative">
//         <Draggable id={2} />
//       </div>
//     </DragDropProvider>
//   );
// };

const Canvas = (props: { children: any }) => {
  const [state, setState] = createStore({
    // scale: 0.6,
    scale: 1,
  });

  return (
    <div class="arena-editor relative w-full h-full p-1px">
      <GridLines width={40} height={30} scale={state.scale()} />
      <div class="arena-canvas-container relative w-full h-full overflow-x-auto overflow-y-auto">
        <div
          class="arena-canvas relative"
          style={{
            // 'transform-origin': 'top left',
            transform: `scale(${state.scale()})`,
          }}
        >
          {props.children}
          {/* <SampleApp /> */}
        </div>
      </div>
    </div>
  );
};

export { Canvas };
