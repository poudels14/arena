import type { Template } from "..";

const metadata: Template.Metadata<{}> = {
  id: "at-grid-layout",
  name: "GridLayout",
  description: "Grid Layout",
  data: {},
};

const GridLayout = (props: Template.Props<{}>) => {
  return (
    <div class="t t-grid-layout w-full h-full" {...props.attrs}>
      <div class="flex flex-col space-y-1">
        <props.Editor.Slot parentId={props.id} />
      </div>
    </div>
  );
};

export default GridLayout;
export { metadata };
