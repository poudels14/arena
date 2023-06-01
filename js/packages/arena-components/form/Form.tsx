import { createStore } from "@arena/solid-store";
import { JSX, createDeferred } from "solid-js";
import { ElementProps } from "./types";
import { NestedObject, NestedObjectProps, StateContext } from "./state";

type FormProps = {
  children: any;
  onSubmit?: (value: any) => void;
} & Pick<ElementProps, "class" | "onChange">;

type NestedForm = ((props: FormProps) => JSX.Element) & {
  Nested: (props: NestedObjectProps) => JSX.Element;
};

const Form: NestedForm = Object.assign(
  (props: FormProps) => {
    const [state, setState] = createStore({});

    createDeferred(
      () => {
        props.onChange?.(state());
      },
      { timeoutMs: 10 }
    );

    const onSubmit = (e: Event) => {
      e.preventDefault();
      props.onSubmit && props.onSubmit(state());
    };

    return (
      <form class={props.class} onSubmit={onSubmit}>
        <StateContext.Provider
          value={{
            state,
            setState,
          }}
        >
          {props.children}
        </StateContext.Provider>
      </form>
    );
  },
  {
    Nested: NestedObject,
  }
);

export { Form };
