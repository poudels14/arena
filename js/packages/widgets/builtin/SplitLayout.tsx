import type { Template } from "..";

const metadata: Template.Metadata<{}> = {
  id: "at-split-layout",
  name: "Split Layout",
  description: "Split Layout",
  data: {},
  class: "bg-white",
};

const SplitLayout = (props: Template.Props<{}>) => {
  return (
    <div
      class="ar-split-layout flex flex-row art-[>.ar-widget](grow) art-[>.slot>.preview](h-full,w-1)"
      {...props.attributes}
    >
      <props.Editor.Slot parentId={props.id} />
    </div>
  );
};

export default SplitLayout;
export { metadata };
