import { onCleanup, untrack, JSX, children } from "solid-js";
import { useAppContext } from "../App";
import type { Widget, WidgetConfig } from "./types";
import { useEditorContext, WidgetDataContext } from "../editor";
import { Store } from "@arena/solid-store";

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

const setWidgetRef = (id: string, node: HTMLElement) => {
  const { getSelectedWidgets, setSelectedWidgets } = useAppContext();

  const onClick = (e: MouseEvent) => {
    const widgets = e.ctrlKey ? untrack(() => [...getSelectedWidgets()]) : [];
    widgets.push({ id, node });
    setSelectedWidgets(widgets);
  };

  node.addEventListener("pointerdown", onClick, {
    capture: true,
  });
  onCleanup(() => node.removeEventListener("pointerdown", onClick));
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
const Widget = (props: WidgetProps) => {
  // TODO(sagar): move this to editor
  const { useWidgetData } = useEditorContext<WidgetDataContext>();

  /**
   * Note(sagar): return proxy here so that Templates don't have to use
   * signals to access `data.{field}`
   */
  const data = new Proxy(
    {},
    {
      get(target: any, fieldName: string) {
        if (!target[fieldName]) {
          target[fieldName] = useWidgetData(
            props.id,
            fieldName,
            props.config.data[fieldName]
          );
        }
        return target[fieldName];
      },
    }
  );

  const widget = {
    id: props.id,
    data,
    attributes: {
      id: props.id,
      ref(node: HTMLElement) {
        setWidgetRef(props.id, node);
      },
      classList: {
        [props.class!]: Boolean(props.class),
      },
    },
  };

  const template = children(() => props.children(widget));

  // TODO(sagar): render inside Widget provider
  // TODO(sagar): make suspense work for this widget
  return <>{template}</>;
};

export { Widget, setWidgetRef };
export type { ActiveWidget };
