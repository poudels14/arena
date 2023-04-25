import { Suspense, createEffect, createContext, useContext } from "solid-js";
import { Widget } from "../widget";
import { useEditorContext } from "./Editor";
import { TemplateStoreContext } from "./withTemplateStore";

const WidgetContext = createContext();

// // TODO(sagar): this is more of a editable widget than DynamicWidget,
// // so, rename it to a better name
// const createDynamicWidget = (widget: Store<Widget>) => {
//   const { useTemplate } = useEditorContext<TemplateStoreContext>();
//   const { Component } = useTemplate(widget.template());
//   createEffect(() => {
//     console.log("WIDGET CREATED:", widget.id());
//   });
//   return (props: any) => (
//     <Suspense fallback={"Loading widget data..."}>
//       <Widget id={widget.id()} config={widget.config} children={Component} />
//     </Suspense>
//   );
// };

// TODO(sagar): this is more of a editable widget than DynamicWidget,
// so, rename it to a better name
const DynamicWidget = (props: { widgetId: string }) => {
  const { useTemplate, useWidgetById } =
    useEditorContext<TemplateStoreContext>();
  const widget = useWidgetById(props.widgetId);

  const { Component } = useTemplate(widget.template.id());
  return (
    <WidgetContext.Provider value={{}}>
      <Suspense fallback={"Loading widget data..."}>
        <Widget
          id={props.widgetId}
          config={widget.config}
          children={Component}
          class="px-40"
        />
      </Suspense>
    </WidgetContext.Provider>
  );
};

export { DynamicWidget };
