import { ResourceReturn, createDeferred, createResource } from "solid-js";
import { InternalEditor, Plugin } from "./types";
import { DataSource } from "@arena/widgets/schema/data";
import { useApiContext } from "../../ApiContext";
import { EditorStateContext } from "../withEditorState";
import { Store } from "@arena/solid-store";
import { Widget } from "@arena/widgets";

type WidgetDataContext = {
  useWidgetData: <T>(widgetId: string, field: string) => ResourceReturn<T>;
};

const WIDGET_DATA_SIGNALS = new Map();

function useWidgetData(widgetId: string, field: string) {
  // @ts-expect-error
  const ctx = this as unknown as InternalEditor<
    any,
    EditorStateContext
  >["context"];

  const widget = ctx.useWidgetById(widgetId);
  const accessorId = `${widgetId}/${field}`;
  if (WIDGET_DATA_SIGNALS.has(accessorId)) {
    return WIDGET_DATA_SIGNALS.get(accessorId);
  }

  const appId = ctx.state.app().id;
  const resource = createResource(async () => {
    const config = widget.config.data[field]();
    switch (config.source) {
      case "template":
      case "dynamic": {
        const cfg = config.config;
        switch (cfg.loader) {
          case "@client/json":
            return useInlineDataSource(cfg);
          case "@client/js":
            return useClientJsDataSource(cfg);
          case "@arena/sql/postgres":
          case "@arena/server-function":
            return await useServerFunctionDataSource(appId, widget, field, cfg);
          default:
            throw new Error(
              "Data source not supported: " + JSON.stringify(cfg)
            );
        }
      }
      default:
        throw new Error("Data source not supported: " + config.source);
    }
  });

  /**
   * Note(sagar): manually trigger refetch so that we can control whether to
   * auto-refetch data on config change
   */
  createDeferred(() => {
    void widget.config.data[field]();
    resource[1].refetch();
  });

  WIDGET_DATA_SIGNALS.set(accessorId, resource);

  // TODO(sagar): cache fieldData accessor such that when other widgets
  // access data for a widget, the accessor can be returned
  //   - Can we trigger suspense when a widget access another widget's
  //     data but that widget hasn't be initialized yet or is ready?
  //   -
  return resource;
}

const withWidgetDataLoaders: Plugin<{}, {}, {}> = (config) => (editor) => {
  Object.assign(editor.context, {
    useWidgetData: useWidgetData.bind(editor.context),
  });
};

function useInlineDataSource<T>(config: DataSource<T>["config"]) {
  return config.value;
}

async function useClientJsDataSource(config: DataSource<any>["config"]) {
  // TODO(sagar): load widget data
  throw new Error("not implemented");
}

async function useServerFunctionDataSource(
  appId: string,
  widget: Store<Widget>,
  field: string,
  config: DataSource<any>["config"]
) {
  const { routes } = useApiContext();
  return await routes.queryWidgetData({
    appId,
    widgetId: widget.id(),
    field,
    updatedAt: widget.updatedAt(),
    params: {}, // TODO(sagar)
  });
}

export { withWidgetDataLoaders };
export type { WidgetDataContext };
