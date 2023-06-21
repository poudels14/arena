import type { Template } from "..";

const metadata: Template.Metadata<{ text: string }> = {
  id: "at-h3",
  name: "Heading 3",
  description: "Heading 3",
  data: {
    text: {
      title: "Text",
      source: "dynamic",
      default: {
        loader: "@client/json",
        value: "Heading 3",
      },
      preview: "Heading 3",
    },
  },
  class: "bg-white",
};

const Heading3 = (props: Template.Props<{ text: string }>) => {
  return (
    <h3 class="text-lg font-medium tracking-tight" {...props.attrs}>
      {props.text}
    </h3>
  );
};

export default Heading3;
export { metadata };
