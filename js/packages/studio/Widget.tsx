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
  WidgetPropsContext,
  TemplateStoreContext,
  useEditorContext,
  WidgetDataContext,
} from "./editor";

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
    if (!ctx.isViewOnly()) {
      e.stopPropagation();
      ctx.setSelectedWidgets([id], !e.ctrlKey);
    }
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
  dataLoaders: [string, ResourceReturn<any>[0]][];
  children: (widget: { attrs: Record<string, any> }) => JSX.Element;
};

/**
 * Using <Widget .../> directly in an app is less preferred than
 * using {@link createWidget}
 */
const WidgetRenderer = (props: WidgetProps) => {
  const ctx = useEditorContext<WidgetDataContext & WidgetPropsContext>();

  const [isDataReady, setIsDataReady] = createSignal(false);
  createComputed(() => {
    const loading = props.dataLoaders.reduce((loading, loader) => {
      const w = loader[1];
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

  const dataLoaders = Object.fromEntries(props.dataLoaders);
  const dataSetters = Object.fromEntries(
    props.dataLoaders.map(([field]) => {
      return [
        `set` + field[0].toUpperCase() + field.substring(1),
        (value: any) => {
          ctx.useWidgetDataSetter(props.id, field)(value);
        },
      ];
    })
  );

  const widget = new Proxy(
    {
      ...dataSetters,
      Editor: ctx.Editor,
      id: props.id,
      attrs: {
        id: props.id,
        ref(node: HTMLElement) {
          setWidgetRef(props.id, node, ctx);
        },
        get classList() {
          return classList();
        },
      },
      isLoading(field: string) {
        return dataLoaders[field]?.loading || false;
      },
    },
    {
      get(target: any, field: string) {
        if (target[field]) {
          return target[field];
        } else if (dataLoaders[field]) {
          return dataLoaders[field]();
        } else if (field.startsWith("set") && field.length > 3) {
          // catch-all for setter
          return () => {};
        }
      },
      getOwnPropertyDescriptor(_, property) {
        return {
          configurable: true,
          enumerable: true,
          get() {
            return _.get(property);
          },
        };
      },
      ownKeys(target) {
        return [
          ...Reflect.ownKeys(target),
          ...props.dataLoaders.flatMap(([k]) => [
            k,
            `set` + k.charAt(0).toUpperCase() + k.substring(1),
          ]),
        ];
      },
    }
  );

  const Component = props.children;
  return (
    <Show when={isDataReady()}>
      <Component {...widget} />
    </Show>
  );
};

// TODO(sagar): this is more of a editable widget than DynamicWidget,
// so, rename it to a better name

const Widget = (props: { widgetId: string }) => {
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
      const data = loader[1];
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
