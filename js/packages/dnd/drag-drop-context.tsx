import {
  JSX,
  createContext,
  createEffect,
  useContext,
  onCleanup,
} from "solid-js";
import { createStore, Store, StoreSetter } from "@arena/solid-store";
import { Draggable } from "./draggable";

type Id = string | number;

type Coordinates = {
  x: number;
  y: number;
};

type Sensor = {
  id: Id;
  origin: Coordinates;
  current: Coordinates;
  get delta(): Coordinates;
};

type Overlay = {
  node: HTMLElement;
};

type State = {
  active: {
    sensor: Sensor | null;
    draggable: Draggable | null;
    overlay: Overlay | null;
  };
};

type Context = {
  state: Store<State>;
  setState: StoreSetter<State>;
};

const DragDropContext = createContext<Context>();
const useDragDropContext = () => useContext(DragDropContext)!;

type DragEvent = {
  draggable: Draggable;
  // droppable?: Droppable | null;
  overlay?: Overlay | null;
};

type DragEventHandler = (event: DragEvent) => void;

type DragAndDropProviderProps = {
  onDragMove?: DragEventHandler;
  onDragEnd?: DragEventHandler;
  children: JSX.Element;
};

const DragDropProvider = (props: DragAndDropProviderProps) => {
  const [state, setState] = createStore<State>({
    active: {
      sensor: null,
      draggable: null,
      overlay: null,
    },
  });

  const pointerUpHandler = (e: PointerEvent) => {
    const active = state.active();
    if (!active.draggable) {
      return;
    }

    const origin = active.sensor!.origin!;
    setState("active", "sensor", "current", {
      x: origin.x,
      y: origin.y,
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

  document.addEventListener("pointerup", pointerUpHandler);
  document.addEventListener("pointercancel", pointerUpHandler);
  document.addEventListener("pointermove", pointerMoveHandler);

  createEffect(() => {
    const node = state.active.overlay.node()!;
    if (!node) return;
    const { style } = node;
    style.setProperty("user-select", "none");
    style.setProperty("transition-timing-function", "ease");
    onCleanup(() => {
      style.removeProperty("user-select");
      style.removeProperty("transition-timing-function");
    });
  });

  createEffect(() => {
    const delta = state.active.sensor.delta();
    const overlay = state.active.overlay();
    if (!delta || !overlay) return;
    overlay.node.style.setProperty(
      "transform",
      `translate3d(${delta.x}px, ${delta.y}px, 0)`
    );
  });

  return (
    <DragDropContext.Provider value={{ state, setState }}>
      {props.children}
    </DragDropContext.Provider>
  );
};

export { DragDropProvider, useDragDropContext };
export type { DragEvent, DragEventHandler };
