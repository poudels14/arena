import {
  onCleanup,
  JSX,
  createMemo,
  ErrorBoundary,
  ResourceReturn,
  createSignal,
  createComputed,
  untrack,
  Show,
} from "solid-js";
import type { Widget, WidgetConfig } from "@arena/widgets/schema";
import { Store } from "@arena/solid-store";
import {
  EditorContext,
  TemplateStoreContext,
  useEditorContext,
  WidgetDataContext,
} from "./editor";
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
    ctx.setSelectedWidgets([id], !e.ctrlKey);
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
  config: Store<WidgetConfig>;

  /**
   * Data loader resource for each field
   *
   * It's setup outside ErrorBoundary so that we can refetch data and reset
   * error state later
   */
  dataLoaders: [string, ResourceReturn<any>][];
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
  const data = new Proxy(Object.fromEntries(props.dataLoaders), {
    get(target: any, fieldName: string) {
      const [data] = target[fieldName] || [];
      return data?.();
    },
  });

  const [isDataReady, setIsDataReady] = createSignal(false);
  createComputed(() => {
    const loading = props.dataLoaders.reduce((loading, loader) => {
      const w = loader[1][0];
      return loading || w.loading;
    }, false);

    untrack(() => {
      !isDataReady() && setIsDataReady(!loading);
    });
  });

  const classList = createMemo(() => {
    const c = props.config.class!()!;
    return {
      "ar-widget": true,
      [c]: Boolean(c),
      "shadow-[inset_0_0px_2px_2px_rgba(229,70,70)]": ctx.isWidgetSelected(
        props.id
      ),
    };
  });

  const widget = {
    id: props.id,
    attributes: {
      id: props.id,
      ref(node: HTMLElement) {
        setWidgetRef(props.id, node, ctx);
      },
      get classList() {
        return classList();
      },
    },
    data,
    // setter for transient data source
    setData(field: string, value: string) {
      ctx.setWidgetData(props.id, field, value);
    },
    Editor: {
      Slot: Slot,
    },
  };

  const Component = props.children;
  return (
    <Show when={isDataReady()}>
      <Component {...widget} />
    </Show>
  );
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
  const ctx = useEditorContext<TemplateStoreContext & WidgetDataContext>();
  const { useTemplate, useWidgetById } = ctx;
  const widget = useWidgetById(props.widgetId);
  const { Component } = useTemplate(widget.template.id());

  const dataLoaders = Object.keys(widget.config.data()).map((fieldName) => {
    return [fieldName, ctx.useWidgetData(props.widgetId, fieldName)] as [
      string,
      ReturnType<typeof ctx.useWidgetData>
    ];
  });

  const hasDataLoadingError = createMemo(() => {
    return dataLoaders.reduce((error, loader) => {
      const data = loader[1][0];
      // use data.loading here so that when it's changed,
      // the error state is re-calculated
      void data.loading;
      return error || data.error;
    }, false);
  });

  return (
    <ErrorBoundary
      fallback={(error, reset) => {
        console.error(error);
        createComputed<boolean>((wasError) => {
          const isError = hasDataLoadingError();
          if (wasError && !isError) {
            reset();
          }
          return isError;
        }, hasDataLoadingError());
        return (
          <div
            class="px-10 py-4 text-red-600 space-y-2"
            ref={(node: HTMLElement) => {
              setWidgetRef(props.widgetId, node, ctx);
            }}
          >
            <div>Error loading data</div>
            <div>{error.toString()}</div>
          </div>
        );
      }}
    >
      <WidgetRenderer
        id={props.widgetId}
        config={widget.config}
        dataLoaders={dataLoaders}
        children={Component}
      />
    </ErrorBoundary>
  );
};

export { Widget };
export type { ActiveWidget };
