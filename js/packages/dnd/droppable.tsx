import { createMemo, createSignal } from "solid-js";
import { useDragDropContext } from "./drag-drop-context";

type Droppable = {
  id: string | number;
  node: HTMLElement;
  /**
   * Set DOM element reference
   */
  ref: (node: HTMLElement) => void;
  data?: any;
  isActiveDroppable: boolean;
};

const createDroppable = (id: string, data?: any) => {
  const { state, setState } = useDragDropContext();

  const isActiveDroppable = createMemo(() => {
    return state.active.collision.droppable()?.id === id;
  });
  const [getNode, setNode] = createSignal<HTMLElement | null>(null);

  const droppable = Object.defineProperties(
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
      isActiveDroppable: {
        get: isActiveDroppable,
      },
    }
  ) as unknown as Droppable;

  setState("droppables", (droppables) => {
    return [...droppables, droppable];
  });

  return droppable;
};

export { createDroppable };
export type { Droppable };
