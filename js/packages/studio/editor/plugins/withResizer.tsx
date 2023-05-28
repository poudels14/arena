import { createSignal, onCleanup, createEffect, For, Show } from "solid-js";
import { createPopper } from "@popperjs/core";
import { Plugin } from "./types";
import { InlineIcon } from "@arena/components";
import DragHandle from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/drag-handle-horizontal";
import "./resizer.css";
import { useEditorContext } from "../editor";

const RESIZER_OFFSET = 1 /* EFFECTIVE_GAP */ + 1; /* BORDER_THICKNESS */

const Resizer = (props: { widgetId: string; node: HTMLElement }) => {
  const { useWidgetById } = useEditorContext();
  const widget = useWidgetById(props.widgetId);

  let resizerRef: any, resizerTarget: any;
  let popper: any;
  const resizer = createResizer();
  createEffect(() => {
    resizerTarget =
      props.node || document.querySelector(`[data-id=${props.widgetId}]`)!;
    popper = createPopper(resizerTarget, resizerRef, {
      placement: "top-start",
      modifiers: [
        {
          name: "flip",
          enabled: false,
        },
        {
          name: "offset",
          options: {
            offset: [-RESIZER_OFFSET, RESIZER_OFFSET],
          },
        },
      ],
    });

    const resizeObserver = new ResizeObserver(() => {
      resizer.update(resizerTarget);
      popper.update();
    });
    resizeObserver.observe(resizerTarget);

    onCleanup(() => {
      resizeObserver.disconnect();
      popper?.destroy();
      resizerTarget = null;
    });
  });

  const [getDragStart, setDragStart] = createSignal<any>();
  const onDragStart = (e: DragEvent) => {
    if (e.currentTarget) {
      e.stopPropagation();

      e.dataTransfer!.effectAllowed = "none";
      const img = document.createElement("img");
      e.dataTransfer!.setDragImage(img, 0, 0);
      setDragStart({
        clientX: e.clientX,
        clientY: e.clientY,
      });
    }
  };

  const onDrag = (e: DragEvent) => {
    const moveDir = (e?.target as HTMLElement)?.dataset["moveDir"];
    // Note(sagar): when drag ends, onDrag receives event with clientX/Y = 0
    //              don't update resizer preview when that happens
    if (e.clientX !== 0) {
      resizer.preview({
        moveDir,
        offsetX: e.clientX - getDragStart()?.clientX,
        offsetY: e.clientY - getDragStart()?.clientY,
      });
    }
  };

  // const onDragEnd = (e: DragEvent) => {
  //   e.stopPropagation();
  //   resizer.resetPreview();
  //   const moveDir = (e?.target as HTMLElement)?.dataset["moveDir"];
  //   resizeWidget({
  //     widgetEle: resizerTarget,
  //     state: activeWidgetState!,
  //     getDragStart,
  //     dragEnd: {
  //       clientX: e.clientX,
  //       clientY: e.clientY,
  //       dir: moveDir,
  //     },
  //   });
  // };

  return (
    <div
      id="widget-resizer"
      class="widget-resizer relative"
      // onDragStart={onDragStart}
      // onDrag={onDrag}
      // onDragEnd={onDragEnd}
      ref={resizerRef}
    >
      <div class="absolute -bottom-[100%] mb-px text-accent-1 bg-[rgb(229,70,70)] rounded">
        <div class="flex flex-row w-52 h-6 items-center text-xs space-x-2">
          <div class="flex pl-2">
            <InlineIcon class="inline-block cursor-move" size="12px">
              <path d={DragHandle[0]} />
            </InlineIcon>
          </div>
          <div>{widget.name()}</div>
        </div>
      </div>
      <div class="left" draggable={true} data-move-dir="left"></div>
      <div class="right" draggable={true} data-move-dir="right"></div>
      <div class="top" draggable={true} data-move-dir="top"></div>
      <div class="bottom" draggable={true} data-move-dir="bottom"></div>
    </div>
  );
};

const ResizerContainer = () => {
  const { isViewOnly, getSelectedWidgets, useWidgetNode } = useEditorContext();
  return (
    <Show when={!isViewOnly()}>
      <div class="resizer-container relative">
        <For each={getSelectedWidgets()}>
          {(widgetId) => {
            const node = useWidgetNode(widgetId)!;
            return (
              <Show when={node()}>
                <Resizer widgetId={widgetId} node={node()!} />
              </Show>
            );
            return;
          }}
        </For>
      </div>
    </Show>
  );
};

const withResizer: Plugin<{}, {}, {}> = (config) => (editor) => {
  editor.components.push(ResizerContainer);
};

const createResizer = () => {
  let previewState = false;
  return {
    preview: ({ moveDir, offsetX, offsetY }: any) => {
      previewState = true;
      const ele = document.querySelector("#widget-resizer") as HTMLElement;
      switch (moveDir) {
        case "left":
          ele?.style?.setProperty("--resizer-offset-x", `${offsetX}px`);
          break;
        case "right":
          ele?.style?.setProperty("--resizer-offset-width", `${offsetX}px`);
          break;
        case "top":
          ele?.style?.setProperty("--resizer-offset-y", `${offsetY}px`);
          break;
        case "bottom":
          ele?.style?.setProperty("--resizer-offset-height", `${offsetY}px`);
          break;
        default:
      }
    },
    resetPreview: () => {
      const ele = document.querySelector("#widget-resizer") as HTMLElement;
      ele?.style?.removeProperty("--resizer-offset-x");
      ele?.style?.removeProperty("--resizer-offset-y");
      ele?.style?.removeProperty("--resizer-offset-width");
      ele?.style?.removeProperty("--resizer-offset-height");
      previewState = false;
    },
    update: (target: HTMLElement) => {
      if (previewState || !target) {
        return;
      }
      const ele = document.querySelector("#widget-resizer") as HTMLElement;
      const coords = target?.getBoundingClientRect();
      const x = target.offsetLeft;
      const y = target.offsetTop;
      const width = coords.width;
      const height = coords.height;

      ele?.style?.setProperty("--x", `${x}px`);
      ele?.style?.setProperty("--y", `${y}px`);
      ele?.style?.setProperty(
        "--resizer-content-width",
        `${width + 2 * RESIZER_OFFSET}px`
      );
      ele?.style?.setProperty(
        "--resizer-content-height",
        `${height + 2 * RESIZER_OFFSET}px`
      );
      ele?.style?.setProperty("--display", "block");
    },
  };
};

const BASE_WIDTH = 50,
  BASE_HEIGHT = 30;
type ResizeWidgetRequest = {
  widgetEle: HTMLElement;
  state: {
    style?: any;
    setStyle: (style: any) => void;
  };
  dragEnd: any;
  getDragStart: () => any;
};
const resizeWidget = (req: ResizeWidgetRequest) => {
  if (!req.widgetEle) {
    return;
  }
  const { style = {}, setStyle } = req.state;

  const dragStart = req.getDragStart();
  const { dragEnd } = req;

  const deltaX = dragEnd.clientX - dragStart.clientX;
  const deltaY = dragEnd.clientY - dragStart.clientY;

  const { width, height } = req.widgetEle.getBoundingClientRect();

  const currColStart = parseInt(style["--n-widget-col-start"] || "1");
  const currRowStart = parseInt(style["--n-widget-row-start"] || "1");
  const currColSpan = parseInt(
    style["--n-widget-col-span"] || Math.floor(width / BASE_WIDTH)
  );
  const currRowSpan = parseInt(
    style["--n-widget-row-span"] || Math.floor(height / BASE_HEIGHT)
  );

  let newColStart = currColStart,
    newRowStart = currRowStart,
    newColSpan = currColSpan,
    newRowSpan = currRowSpan;

  switch (dragEnd.dir) {
    case "left":
      newColStart = currColStart + Math.floor(deltaX / BASE_WIDTH);
      break;
    case "right":
      newColSpan = currColSpan + Math.floor(deltaX / BASE_WIDTH);
      break;
    case "top":
      newRowStart = currColSpan + Math.floor(deltaY / BASE_HEIGHT);
      break;
    case "bottom":
      newRowSpan = currRowSpan + Math.floor(deltaY / BASE_HEIGHT);
      break;
  }

  const finalStyle = {
    ...style,
    "--n-widget-col-start": newColStart,
    "--n-widget-row-start": newRowStart,
    "--n-widget-col-span": Math.max(1, newColSpan),
    "--n-widget-row-span": Math.max(1, newRowSpan),
    width: "var(--n-widget-width)",
    height: "var(--n-widget-height)",
  };
  setStyle(finalStyle);
};

export { withResizer };
