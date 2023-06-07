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

const Heading3 = (
  props: Template.Props<{ type: string; placeholder: string; value: string }>
) => {
  return (
    <input
      {...props.attributes}
      class="px-2 py-1 rounded border border-accent-6 outline-none ring-inset focus:ring-1 placeholder:text-accent-9"
      type={props.data.type}
      value={props.data.value}
      placeholder={props.data.placeholder}
      onInput={(e) => {
        props.setData("value", e.target.value);
      }}
    />
  );
};

export default Heading3;
export { metadata };
