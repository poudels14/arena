import { Store, StoreSetter } from "@arena/solid-store";
import { createContext, useContext } from "solid-js";

type StateContext<T> = {
  state: Store<T>;
  setState: StoreSetter<T>;
};

const StateContext = createContext<StateContext<any>>();

type NestedObjectProps = {
  key: string;
  children: any;
};

const NestedObject = (props: NestedObjectProps) => {
  const { state, setState } = useContext(StateContext)!;
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

export { NestedObject, StateContext };
export type { NestedObjectProps };
