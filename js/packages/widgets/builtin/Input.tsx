import type { Template } from "..";

const metadata: Template.Metadata<{
  type: string;
  placeholder: string;
  value: string;
}> = {
  id: "at-input",
  name: "Input",
  description: "Input",
  data: {
    type: {
      title: "Type",
      source: "userinput",
      default: {
        value: "text",
      },
      preview: "text",
    },
    placeholder: {
      title: "Placeholder",
      source: "userinput",
      default: {
        value: "Enter text",
      },
      preview: "Enter text",
    },
    value: {
      title: "Input",
      source: "transient",
    },
  },
  class: "bg-white",
};

const Input = (
  props: Template.Props<{ type: string; placeholder: string; value: string }>
) => {
  return (
    <input
      {...props.attrs}
      class="px-2 py-1 rounded border border-accent-6 outline-none ring-inset focus:ring-1 placeholder:text-accent-9"
      type={props.type}
      value={props.value}
      placeholder={props.placeholder}
      onInput={(e) => {
        props.setValue(e.target.value);
      }}
    />
  );
};

export default Input;
export { metadata };
