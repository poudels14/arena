import { $RAW } from "@arena/solid-store";
import { batch, createEffect, createSignal, onCleanup } from "solid-js";
import { useDragDropContext } from "./drag-drop-context";

type Draggable = {
  id: string | number;
  node: HTMLElement;
  /**
   * Set DOM element reference
   */
  ref: (node: HTMLElement) => void;
  data?: any;
  isActiveDraggable: boolean;
};

const createDraggable = (id: string, data?: any) => {
  const { state, setState } = useDragDropContext();

  const isActiveDraggable = () => state.active.draggable.id() === id;
  const [getNode, setNode] = createSignal<HTMLElement | null>(null);

  let draggable = Object.defineProperties(
    {},
    {
      node: {
        get: getNode,
      },
      ref: {
        get: getNode,
        set: setNode,
      },
      id: {
        value: id,
      },
      data: {
        value: data,
      },
      isActiveDraggable: {
        get: isActiveDraggable,
      },
    }
  ) as unknown as Draggable;

  createEffect(() => {
    const handler = createPointerDownHandler(setState, draggable);
    const node = getNode();
    node?.addEventListener("pointerdown", handler);
    onCleanup(() => node?.removeEventListener("pointerdown", handler));
  });

  return draggable;
};

const createPointerDownHandler =
  (setState: any, draggable: Draggable) => (e: PointerEvent) => {
    batch(() => {
      setState("active", "draggable", draggable);
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
          // Note(sagar): need to check if `this` is a store value
          // or raw object; it becomes raw object when `sensor.delta` is
          // accessed on raw object sensor
          const { current, origin } = this[$RAW] ? this() : this;
          return {
            x: current.x - origin.x,
            y: current.y - origin.y,
          };
        },
      });
    });
  };

export { createDraggable };
export type { Draggable };
