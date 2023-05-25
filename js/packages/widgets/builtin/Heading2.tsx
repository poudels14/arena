import type { Template } from "..";

const metadata: Template.Metadata<{ text: string }> = {
  id: "at-h2",
  name: "Heading 2",
  description: "Heading 2",
  data: {
    text: {
      title: "Text",
      source: "dynamic",
      default: {
        loader: "@client/json",
        value: "Heading 2",
      },
    },
  },
};

const Heading2 = (props: Template.Props<{ text: string }>) => {
  return <h2 {...props.attributes}>{props.data.text}</h2>;
};

export default Heading2;
export { metadata };
