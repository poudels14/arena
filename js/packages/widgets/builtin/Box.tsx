import type { Template } from "..";

const metadata: Template.Metadata<{}> = {
  id: "at-box",
  name: "Box",
  description: "Box element",
  data: {},
  class: "bg-white",
};

const Box = (props: Template.Props<{}>) => {
  return (
    <div {...props.attrs}>
      <props.Editor.Slot parentId={props.id} />
    </div>
  );
};

export default Box;
export { metadata };
