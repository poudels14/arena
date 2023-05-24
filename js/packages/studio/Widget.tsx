import { onCleanup, JSX, children, createMemo, Suspense } from "solid-js";
import type { Widget, WidgetConfig } from "@arena/widgets/schema";
import {
  EditorContext,
  TemplateStoreContext,
  useEditorContext,
  WidgetDataContext,
} from "./editor";
import { Store } from "@arena/solid-store";
import { Slot } from "./Slot";

type ActiveWidget = {
  id?: string | null;
  /**
   * This node will be highlighted in the editor
   * In case of "template" type widgets, there can be more than
   * one nodes with same widgetId and this node prop will determine
   * which one to highlight. If node is missing, it will select the
   * first node with the given widget id
   */
  node?: any | null;
};

const setWidgetRef = (
  id: string,
  node: HTMLElement,
  ctx: EditorContext<any>
) => {
  const onClick = (e: MouseEvent) => {
    e.stopPropagation();
    ctx.setSelectedWidget(id, !e.ctrlKey);
  };

  node.addEventListener("pointerdown", onClick);
  ctx.registerWidgetNode(id, node);
  onCleanup(() => {
    ctx.registerWidgetNode(id, null);
    node.removeEventListener("pointerdown", onClick);
  });
};

type WidgetProps = {
  id: string;
  name?: string;
  // TODO(sagar): remove class since config already has classList?
  class?: string;
  config: Store<WidgetConfig>;
  children: (widget: { attributes: Record<string, any> }) => JSX.Element;
};

/**
 * Using <Widget .../> directly in an app is less preferred than
 * using {@link createWidget}
 */
const WidgetRenderer = (props: WidgetProps) => {
  const ctx = useEditorContext<WidgetDataContext>();

  /**
   * Note(sagar): return proxy here so that Templates don't have to use
   * signals to access `data.{field}`
   */
  const data = new Proxy(
    {},
    {
      get(target: any, fieldName: string) {
        if (!target[fieldName]) {
          target[fieldName] = ctx.useWidgetData(
            props.id,
            fieldName,
            props.config.data[fieldName]
          );
        }
        return target[fieldName]();
      },
    }
  );

  const classList = createMemo(() => {
    const c = props.config.class!()!;
    return {
      "ar-widget": true,
      [props.class!]: Boolean(props.class),
      [c]: Boolean(c),
    };
  });

  const widget = {
    id: props.id,
    data,
    attributes: {
      id: props.id,
      ref(node: HTMLElement) {
        setWidgetRef(props.id, node, ctx);
      },
      get classList() {
        return classList();
      },
    },
    Editor: {
      Slot: Slot,
    },
  };

  const template = children(() => props.children(widget));
  // TODO(sagar): make suspense work for this widget
  return <>{template}</>;
};

// TODO(sagar): this is more of a editable widget than DynamicWidget,
// so, rename it to a better name

/**
 *
 * @param previousWidgetId is the id of the widget that's rendered before this
 * widget in a Slot. it is used to determine the order of widgets
 */
const Widget = (props: {
  widgetId: string;
  previousWidgetId?: string | null;
}) => {
  const { useTemplate, useWidgetById } =
    useEditorContext<TemplateStoreContext>();
  const widget = useWidgetById(props.widgetId);
  const { Component } = useTemplate(widget.template.id());
  return (
    <Suspense fallback={"Loading widget data..."}>
      <WidgetRenderer
        id={props.widgetId}
        config={widget.config}
        children={Component}
        class="px-40"
      />
    </Suspense>
  );
};

export { Widget };
export type { ActiveWidget };
