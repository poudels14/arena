import type { Template } from "..";

const metadata: Template.Metadata<{ text: string }> = {
  id: "at-h3",
  name: "Heading 3",
  description: "Heading 3",
  data: {
    text: {
      title: "Text",
      dataSource: {
        type: "dynamic",
        default: {
          source: "inline",
          value: "Heading 3",
        },
      },
    },
  },
};

const Heading3 = (props: Template.Props<{ text: string }>) => {
  return <h3 {...props.attributes}>{props.data.text}</h3>;
};

export default Heading3;
export { metadata };
