import type { Template } from "..";

const metadata: Template.Metadata<{ text: string }> = {
  id: "at-h1",
  name: "Heading 1",
  description: "Heading 1",
  data: {
    text: {
      title: "Text",
      dataSource: {
        type: "dynamic",
        default: {
          source: "inline",
          value: "Heading 1",
        },
      },
    },
  },
};

const Heading1 = (props: Template.Props<{ text: string }>) => {
  return <h1 {...props.attributes}>{props.data.text}</h1>;
};

export default Heading1;
export { metadata };
