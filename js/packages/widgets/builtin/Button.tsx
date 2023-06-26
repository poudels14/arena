import type { Template } from "..";

const metadata: Template.Metadata<{
  label: string;
}> = {
  id: "at-button",
  name: "Button",
  description: "Button",
  data: {
    label: {
      title: "Label",
      source: "userinput",
      default: {
        value: "Button",
      },
      preview: "Button",
    },
  },
  class: "bg-brand-12/90 hover:bg-brand-12/80 text-white",
};

const Button = (
  props: Template.Props<{
    label: string;
  }>
) => {
  return (
    <button
      class="px-2 py-1 rounded border border-accent-6 outline-none focus:ring-1"
      type="button"
      {...props.attrs}
    >
      {props.label}
    </button>
  );
};

export default Button;
export { metadata };
