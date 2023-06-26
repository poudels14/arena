import { splitProps, useContext } from "solid-js";
import { useStateContext } from "./state";
import { ElementProps } from "./types";

type InputProps = {
  type?: string;
  placeholder?: string;
} & ElementProps;

export default function Input(props: InputProps) {
  const { setState } = useStateContext<any>();
  // set initial value
  setState(props.name, props.value);
  const [attrs] = splitProps(props, ["name", "value", "type", "placeholder"]);

  return (
    <input
      {...attrs}
      class="px-2 py-1 rounded border border-accent-6 outline-none ring-inset focus:ring-1 placeholder:text-accent-9"
      classList={{
        [props.class!]: Boolean(props.class),
      }}
      onInput={(e) => {
        const value = e.target.value;
        props.onChange?.(value);
        setState(props.name, value);
      }}
    />
  );
}
