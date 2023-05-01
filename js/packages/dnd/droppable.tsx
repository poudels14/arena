import { createSignal } from "solid-js";
import { useDragDropContext } from "./drag-drop-context";

type Droppable = {
  id: string | number;
  node: HTMLElement;
  data?: any;
  isActiveDroppable: boolean;
};

const createDroppable = (id: string, data?: any) => {
  const { state, setState } = useDragDropContext();

  const isActiveDroppable = () => {
    return state.active.collision.droppable()?.id === id;
  };
  const [node, setNode] = createSignal<HTMLElement | null>(null);

  const droppable = Object.defineProperties(
    (node: HTMLElement) => {
      setNode(node);
    },
    {
      node: {
        get: node,
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
    droppables.push(droppable);
    return droppables;
  });

  return droppable;
};

export { createDroppable };
export type { Droppable };
