import {
  JSX,
  createContext,
  createEffect,
  useContext,
  onCleanup,
  untrack,
} from "solid-js";
import { createStore, Store, StoreSetter } from "@arena/solid-store";
import { Draggable } from "./draggable";
import { Droppable } from "./droppable";
import { findDroppableWithClosestCenter } from "./collisions/closest-center";
import { Collision, Sensor } from "./types";
import { Overlay } from "./overlay";

type State = {
  active: {
    sensor: Sensor | null;
    draggable: Draggable | null;
    collision: Collision | null;
    overlay: Overlay | null;
  };
  droppables: Droppable[];
};

type Context = {
  state: Store<State>;
  setState: StoreSetter<State>;
};

const DragDropContext = createContext<Context>();
const useDragDropContext = () => useContext(DragDropContext)!;

type DragEvent = {
  draggable: Draggable;
  overlay?: Overlay | null;
};

type DragEventHandler = (event: DragEvent) => void;

type DragEndEvent = {
  draggable: Draggable;
  droppable: Droppable | null;
};

type DragEndHandler = (e: DragEndEvent) => void;

type DragAndDropProviderProps = {
  onDragMove?: DragEventHandler;
  onDragEnd?: DragEndHandler;
  children: JSX.Element;
};

const DragDropProvider = (props: DragAndDropProviderProps) => {
  const [state, setState] = createStore<State>({
    active: {
      sensor: null,
      draggable: null,
      collision: null,
      overlay: null,
    },
    droppables: [],
  });

  const dragEndHandler = (_: PointerEvent) => {
    const active = untrack(() => state.active());
    // Note(sagar): since drag end handler is attached to the document,
    // only act on it if draggable isn't null
    if (!active.draggable) {
      return;
    }
    props.onDragEnd?.({
      draggable: active.draggable!,
      droppable: active.collision?.droppable || null,
    });
    setState("active", "sensor", null);
    setState("active", "draggable", null);
  };

  const pointerMoveHandler = (e: PointerEvent) => {
    const active = state.active();
    if (!active.draggable) {
      return;
    }
    setState("active", "sensor", "current", {
      x: e.clientX,
      y: e.clientY,
    });
  };

  document.addEventListener("pointerup", dragEndHandler);
  document.addEventListener("pointercancel", dragEndHandler);
  document.addEventListener("pointerleave", dragEndHandler);
  document.addEventListener("pointermove", pointerMoveHandler);

  createEffect(() => {
    const draggable = state.active.draggable.node();
    const node = state.active.overlay.node() || draggable;
    if (!node) return;
    const { style } = node;
    if (draggable) {
      style.setProperty("user-select", "none");
      style.setProperty("transition-timing-function", "ease");
      style.setProperty("z-index", "99999999");
    }
    onCleanup(() => {
      style.removeProperty("user-select");
      style.removeProperty("transition-timing-function");
      style.removeProperty("z-index");
      style.removeProperty("transform");
    });
  });

  createEffect(() => {
    const delta = state.active.sensor.delta();
    const overlay = state.active.overlay() || state.active.draggable();
    if (!delta || !overlay) return;
    overlay.node.style.setProperty(
      "transform",
      `translate3d(${delta.x}px, ${delta.y}px, 0)`
    );
    onCleanup(() => overlay.node.style.removeProperty("transform"));
  });

  // detect collisions
  createEffect(() => {
    const sensor = state.active.sensor();
    if (!sensor) {
      setState("active", "collision", null);
      return;
    }
    const droppables = state.droppables();
    const collision = findDroppableWithClosestCenter(sensor, droppables);
    setState("active", "collision", collision);
  });

  return (
    <DragDropContext.Provider value={{ state, setState }}>
      {props.children}
    </DragDropContext.Provider>
  );
};

export { DragDropProvider, useDragDropContext };
export type { DragEvent, DragEventHandler, DragEndEvent, Sensor };
