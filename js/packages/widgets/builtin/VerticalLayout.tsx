import type { Template } from "..";

const metadata: Template.Metadata<{}> = {
  id: "at-vertical-layout",
  name: "Vertical Layout",
  description: "Vertical Layout",
  data: {},
  class: "bg-white",
};

const VerticalLayout = (props: Template.Props<{}>) => {
  return (
    <div
      class="ar-vertical-layout w-full flex flex-col art-[>.slot>.preview](h-1,w-auto)"
      {...props.attributes}
    >
      <props.Editor.Slot parentId={props.id} />
    </div>
  );
};

export default VerticalLayout;
export { metadata };
