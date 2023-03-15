export { DragDropProvider } from "./drag-drop-context";
export type { DragEvent, DragEventHandler } from "./drag-drop-context";
export { createDraggable } from "./draggable";

declare module "solid-js" {
  namespace JSX {
    interface Directives {
      draggable: boolean;
    }
  }
}
