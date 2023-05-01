import { Draggable } from "./draggable";
import { Droppable } from "./droppable";

export { DragDropProvider, useDragDropContext } from "./drag-drop-context";
export type {
  DragEvent,
  DragEventHandler,
  DragEndEvent,
} from "./drag-drop-context";
export { createDraggable } from "./draggable";
export { createDroppable } from "./droppable";
export { DragOverlay } from "./overlay";

declare module "solid-js" {
  namespace JSX {
    interface Directives {
      "use:draggable": Draggable;
      draggable: Draggable;
      droppable: Droppable;
      "use:droppable": Droppable;
    }
  }
}
