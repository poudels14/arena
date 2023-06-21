import type { Template } from "..";

const metadata: Template.Metadata<{}> = {
  id: "at-layout",
  name: "Layout",
  description: "Layout",
  data: {},
};

const Layout = (props: Template.Props<{}>) => {
  // TODO(sagar): create separate output of templates for view/edit mode
  // The view mode should be optimized and shouldn't contain code that is
  // needed only in edit mode
  let ref: any;
  return (
    <div class="t t-layout-vertical h-full" {...props.attrs}>
      <div class="flex flex-col space-y-1">
        <props.Editor.Slot parentId={props.id} />
      </div>
    </div>
  );
};

export default Layout;
export { metadata };
