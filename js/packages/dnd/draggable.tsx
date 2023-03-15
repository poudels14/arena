import { batchUpdates } from "@arena/solid-store";
import { createEffect, createSignal, onCleanup, untrack } from "solid-js";
import { useDragDropContext } from "./drag-drop-context";

type Draggable = {
  id: string | number;
  node: HTMLElement;
  isActiveDraggable: boolean;
};

const createDraggable = (id: string) => {
  const { state, setState } = useDragDropContext();

  const isActiveDraggable = () => state.active.draggable.id() === id;
  const [node, setNode] = createSignal<HTMLElement | null>(null);

  const pointerDownHandler = (e: PointerEvent) => {
    setState("active", "sensor", {
      id: "pointerdown",
      origin: {
        x: e.clientX,
        y: e.clientY,
      },
      current: {
        x: e.clientX,
        y: e.clientY,
      },
      get delta() {
        const current = this.current();
        const origin = this.origin();
        return {
          x: current.x - origin.x,
          y: current.y - origin.y,
        };
      },
    });

    batchUpdates(() => {
      const draggableNode = untrack(() => node())!;
      setState("active", "overlay", {
        // TODO(sagar): use overlay node if overlay is used
        node: draggableNode,
      });

      setState("active", "draggable", {
        id,
        node: draggableNode,
      });
    });
  };

  const d = Object.defineProperties(
    (node: HTMLElement) => {
      setNode(node);
      createEffect(() => {
        node.addEventListener("pointerdown", pointerDownHandler);
        onCleanup(() =>
          node.removeEventListener("pointerdown", pointerDownHandler)
        );
      });
    },
    {
      ref: {
        enumerable: true,
        value: (node: HTMLElement | null) => {
          setNode(node);
        },
      },
      node: {
        get: node,
      },
      id: {
        value: id,
      },
      isActiveDraggable: {
        get: isActiveDraggable,
      },
    }
  ) as unknown as Draggable;

  return d;
};

export { createDraggable };
export type { Draggable };
