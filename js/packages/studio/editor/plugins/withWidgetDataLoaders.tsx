import {
  ResourceReturn,
  createMemo,
  createReaction,
  createResource,
  createSignal,
  startTransition,
  untrack,
} from "solid-js";
import isEqual from "fast-deep-equal/es6";
import { klona } from "klona";
import { InternalEditor, Plugin } from "./types";
import { DataSource } from "@arena/widgets/schema/data";
import { useApiContext } from "../../ApiContext";
import { EditorStateContext } from "../withEditorState";
import { $RAW, Store } from "@arena/solid-store";
import { Widget } from "@arena/widgets";

type WidgetDataContext = {
  useWidgetData: <T>(widgetId: string, field: string) => ResourceReturn<T>;

  /**
   * Only for data source of type "transient"
   */
  setWidgetData: (widgetId: string, field: string, value: any) => void;
};

const withWidgetDataLoaders: Plugin<{}, {}, {}> = (config) => (editor) => {
  Object.assign(editor.context, {
    useWidgetData: useWidgetData.bind(editor.context),
    setWidgetData(widgetId: string, field: string, value: any) {
      if (TRANSIENT_DATA_STORE.has(widgetId, field)) {
        TRANSIENT_DATA_STORE.get(widgetId, field)[1](value);
        return;
      }

      const ctx = this as unknown as InternalEditor<
        any,
        EditorStateContext
      >["context"];

      const widget = ctx.useWidgetById(widgetId);
      const fieldConfig = untrack(widget.config.data[field]);
      // If a new data field is added to widget template,
      // existing widget instances will be missing the new field
      if (!fieldConfig) {
        console.warn(
          `Widget [${widgetId}] doesn't support data field: ${field}`
        );
        return;
      }
      if (fieldConfig.source == "transient") {
        TRANSIENT_DATA_STORE.get(widgetId, field)[1](value);
      } else if (
        fieldConfig.source == "config" &&
        !isEqual(fieldConfig.config, value)
      ) {
        ctx.updateWidget(widgetId, "config", "data", field, "config", value);
      }
    },
  });
};

const createTransientDataStore = () => {
  const TRANSIENT_DATA = new Map();
  return {
    get(widgetId: string, field: string, defaultValue?: any) {
      const accessorId = `${widgetId}/${field}`;
      let signal;
      if ((signal = TRANSIENT_DATA.get(accessorId))) {
        return signal;
      }
      signal = createSignal(defaultValue);
      TRANSIENT_DATA.set(accessorId, signal);
      return signal;
    },
    has(widgetId: string, field: string) {
      return TRANSIENT_DATA.has(`${widgetId}/${field}`);
    },
  };
};
const WIDGET_DATA_SIGNALS = new Map();
const TRANSIENT_DATA_STORE = createTransientDataStore();

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

  if (widget.config.data[field][$RAW].source == "transient") {
    return getTransientDataResource(widgetId, field);
  }

  const app = ctx.state.app();
  const fieldConfig = createMemo(
    widget.config.data[field],
    {},
    {
      equals(prev, next) {
        return isEqual(prev, next);
      },
    }
  );

  const propsGenerator = createMemo(() => {
    const config = fieldConfig();

    switch (config.source) {
      case "template":
      case "dynamic":
        const cfg = config.config;
        switch (cfg.loader) {
          case "@arena/sql/postgres":
          case "@arena/server-function":
            if (cfg.metatada?.propsGenerator) {
              const widgets = ctx.state.app.widgets();
              const uniqueWidgetSlugs = new Set();
              let slugStr = "";
              Object.entries(widgets).map(([id, w]) => {
                if (uniqueWidgetSlugs.has(w.slug)) {
                  return;
                }
                uniqueWidgetSlugs.add(w.slug);
                slugStr += `"${id}": ${w.slug},`;
              });
              return new Function(
                `{ ${slugStr} }`,
                cfg.metatada.propsGenerator
              );
            }
          default:
            return () => ({});
        }
      default:
        return () => ({});
    }
  });

  const getProps = createMemo(() => {
    let generator = propsGenerator();
    if (generator) {
      // TODO(sagar): clean up this proxy
      const genCtxt = new Proxy(
        {},
        {
          get(target, widgetId) {
            return new Proxy(
              {},
              {
                get(target, field) {
                  // @ts-expect-error
                  return ctx.useWidgetData(widgetId, field)?.[0]?.();
                },
              }
            );
          },
        }
      );
      const p = generator(genCtxt);
      return p;
    }
  });

  const resource = createResource(getProps, async (props) => {
    const config = fieldConfig();
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
            return await useServerFunctionDataSource(
              app.id,
              widget,
              field,
              cfg,
              props
            );
          default:
            throw new Error(
              "Data source not supported: " + JSON.stringify(cfg)
            );
        }
      }
      case "config":
        return klona(config.config);
      case "userinput":
        return config.config.value;
      case "transient":
        throw new Error("unreachable");
      default:
        // @ts-expect-error
        throw new Error("Data source not supported: " + config.source);
    }
  });

  /**
   * Note(sagar): manually trigger refetch so that we can control whether to
   * auto-refetch data on config change
   */
  const track = createReaction(() => {
    if (!widget()) {
      WIDGET_DATA_SIGNALS.delete(accessorId);
      return;
    }
    startTransition(() => resource[1].refetch());
    track(fieldConfig);
  });
  track(fieldConfig);

  WIDGET_DATA_SIGNALS.set(accessorId, resource);

  // TODO(sagar): cache fieldData accessor such that when other widgets
  // access data for a widget, the accessor can be returned
  //   - Can we trigger suspense when a widget access another widget's
  //     data but that widget hasn't be initialized yet or is ready?
  //   -
  return resource;
}

function getTransientDataResource(widgetId: string, field: string) {
  return [
    TRANSIENT_DATA_STORE.get(widgetId, field)[0],
    {
      loading: false,
    },
  ];
}

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
  config: DataSource<any>["config"],
  props: any
) {
  const { routes } = useApiContext();
  return await routes.queryWidgetData({
    appId,
    widgetId: widget.id(),
    field,
    updatedAt: widget.updatedAt(),
    props,
  });
}

export { withWidgetDataLoaders };
export type { WidgetDataContext };
