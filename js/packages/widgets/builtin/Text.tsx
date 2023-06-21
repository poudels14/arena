import type { Template } from "..";

const metadata: Template.Metadata<{ text: string }> = {
  id: "at-text",
  name: "Text",
  description: "Text",
  data: {
    text: {
      title: "Text",
      source: "dynamic",
      default: {
        loader: "@client/json",
        value: "Text",
      },
      preview: "Text",
    },
  },
  class: "bg-white",
};

const Text = (props: Template.Props<{ text: string }>) => {
  return (
    <p class="text-accent-12" {...props.attrs}>
      {props.text}
    </p>
  );
};

export default Text;
export { metadata };
