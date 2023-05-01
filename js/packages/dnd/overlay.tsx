import { createEffect, onCleanup } from "solid-js";
import { useDragDropContext } from "./drag-drop-context";

type Overlay = {
  node: HTMLElement;
};

const DragOverlay = () => {
  const { state, setState } = useDragDropContext();
  const setRef = (node: any) => {
    setState("active", "overlay", {
      node,
    });
  };

  createEffect(() => {
    const overlay = state.active.overlay.node()!;
    const draggable = state.active.draggable.node()!;
    const propertiesToClone = [
      "color",
      "background",
      "border",
      "border-radius",
      "font-size",
    ];
    if (draggable) {
      const rect = draggable.getBoundingClientRect();
      const style = window.getComputedStyle(draggable, null);

      overlay.style.setProperty("opacity", "1");
      overlay.style.setProperty("top", rect.top + "px");
      overlay.style.setProperty("left", rect.left + "px");
      overlay.style.setProperty("width", rect.width + "px");
      overlay.style.setProperty("height", rect.height + "px");

      propertiesToClone.forEach((p) =>
        overlay.style.setProperty(p, style.getPropertyValue(p))
      );
    }
    onCleanup(() => {
      ["top", "left", "width", "height"].forEach((p) =>
        overlay.style.removeProperty(p)
      );
      propertiesToClone.forEach((p) => overlay.style.removeProperty(p));
      overlay.style.setProperty("opacity", "0");
    });
  });

  return (
    <div ref={setRef} style="position: fixed; opacity: 0;">
      {state.active.draggable.node()?.cloneNode(true)}
    </div>
  );
};

export { DragOverlay };
export type { Overlay };
