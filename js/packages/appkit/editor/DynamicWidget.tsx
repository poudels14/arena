import { Suspense } from "solid-js";
import { Widget } from "../widget";
import { useEditorContext } from "./Editor";
import { TemplateStoreContext } from "./withTemplateStore";

// TODO(sagar): this is more of a editable widget than DynamicWidget,
// so, rename it to a better name
const DynamicWidget = (props: { widgetId: string }) => {
  const { useTemplate, useWidgetById } =
    useEditorContext<TemplateStoreContext>();
  const widget = useWidgetById(props.widgetId);
  const { Component } = useTemplate(widget.template.id());
  return (
    <Suspense fallback={"Loading widget data..."}>
      <Widget
        id={props.widgetId}
        config={widget.config}
        children={Component}
        class="px-40"
      />
    </Suspense>
  );
};

export { DynamicWidget };
