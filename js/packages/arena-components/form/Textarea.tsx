import { splitProps, useContext } from "solid-js";
import { ElementProps } from "./types";
import { useStateContext } from "./state";

type TextareaProps = {
  type?: string;
  placeholder?: string;
  rows?: number;
} & ElementProps;

export default function Textarea(props: TextareaProps) {
  const { setState } = useStateContext<any>();
  // set initial value
  setState(props.name, props.value);
  const [attrs] = splitProps(props, ["name", "value", "placeholder", "rows"]);

  return (
    <textarea
      {...attrs}
      value={props.value}
      id="comment"
      class="p-2 rounded-md border border-accent-6 outline-none focus:ring-inset focus:ring-1 placeholder:text-accent-9"
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
