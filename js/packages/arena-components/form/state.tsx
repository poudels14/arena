import { Store, StoreSetter, createStore } from "@arena/solid-store";
import { createContext, onCleanup, useContext } from "solid-js";

type StateContext<T> = {
  state: Store<T>;
  setState: StoreSetter<T>;
};

const StateContext = createContext<StateContext<any>>();
function useStateContext<T>(key?: Object) {
  let context: StateContext<T>;
  if ((context = useContext(StateContext)!)) {
    return context;
  }
  const [state, setState] = createStore<T>({} as T);
  onCleanup(() => {
    // TODO(sagar): cleanup store here
  });
  return {
    state,
    setState,
  };
}

type NestedObjectProps = {
  key: string;
  children: any;
};

const NestedObject = (props: NestedObjectProps) => {
  const { state, setState } = useStateContext<any>();
  return (
    <StateContext.Provider
      value={{
        // @ts-expect-error
        state: state![props.key] as Store<any>,
        setState: ((...args: [any]) =>
          setState(props.key, ...args)) as StoreSetter<any>,
      }}
    >
      {props.children}
    </StateContext.Provider>
  );
};

export { NestedObject, StateContext, useStateContext };
export type { NestedObjectProps };
