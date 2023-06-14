import {
  ResourceReturn,
  createMemo,
  createReaction,
  createResource,
  createSignal,
  onCleanup,
  startTransition,
  untrack,
} from "solid-js";
import isEqual from "fast-deep-equal/es6";
import { klona } from "klona";
import { InternalEditor, Plugin } from "./types";
import { DataSource } from "@arena/widgets/schema/data";
import { useApiContext } from "../../ApiContext";
import { EditorStateContext } from "../withEditorState";
import { Widget } from "@arena/widgets";

type WidgetDataContext = {
  useWidgetData: <T>(widgetId: string, field: string) => ResourceReturn<T>[0];

  useWidgetDataSetter: (
    widgetId: string,
    field: string
  ) => (value: any) => void;
};

const withWidgetDataLoaders: Plugin<{}, {}, WidgetDataContext> =
  (config) => (editor) => {
    const ctx = (
      editor as unknown as InternalEditor<
        any,
        EditorStateContext & WidgetDataContext
      >
    ).context;

    const ONCE_CACHE = new Map();
    function once<T extends Object>(key: string, init: () => T) {
      let value: T;
      if (!(value = ONCE_CACHE.get(key))) {
        value = init();
        ONCE_CACHE.set(key, value);
      }
      return value;
    }
    onCleanup(() => ONCE_CACHE.clear());

    const getFieldConfig = (widgetId: string, field: string) => {
      const widget = ctx.useWidgetById(widgetId);
      const fieldConfig = widget.config.data[field]();
      // If a new data field is added to widget template,
      // existing widget instances will be missing the new field
      if (!fieldConfig) {
        console.warn(
          `Widget [${widgetId}] doesn't support data field: ${field}`
        );
        return null;
      }
      return fieldConfig;
    };

    const getFieldConfigMemo = (widgetId: string, field: string) =>
      once(`${widgetId}/${field}`, () =>
        createMemo(
          () => {
            const widget = ctx.useWidgetById(widgetId);
            return klona(widget.config.data[field]());
          },
          {},
          {
            equals(prev: any, next: any) {
              return isEqual(prev, next);
            },
          }
        )
      );

    const getPropsGetterGenerator = (widgetId: string, field: string) => {
      const fieldConfig = getFieldConfigMemo(widgetId, field);
      return createMemo(() => {
        const config = untrack(fieldConfig);
        switch (config.source) {
          case "template":
          case "dynamic":
            const cfg = config.config;
            switch (cfg.loader) {
              case "@arena/sql/postgres":
              case "@arena/server-function":
                if (cfg.metatada?.propsGenerator) {
                  const widgets = untrack(ctx.state.app.widgets);
                  const widget = ctx.useWidgetById(widgetId);
                  // Note(sp): access propsGenerator so that memo is re-calced
                  // when propsGenerator is updated
                  void widget.config.data[
                    field
                    // @ts-expect-error
                  ].config!.metatada.propsGenerator();
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
            }
        }
      });
    };

    const propsGeneratorContext = new Proxy(
      {},
      {
        get(cache: any, widgetId: string) {
          return new Proxy(
            {},
            {
              get(_, field: string) {
                if (field == "toJSON") return;
                let key = `${widgetId}/${field}`;
                let getter: any;
                if (!(getter = cache[key])) {
                  cache[key] = getter = ctx.useWidgetData(widgetId, field);
                }
                return getter?.();
              },
            }
          );
        },
      }
    );

    const getDyanmicDataResource = (
      widgetId: string,
      field: string,
      propsGetter: any
    ) => {
      return createResource(propsGetter, async (props) => {
        const app = ctx.state.app();
        const widget = app.widgets[widgetId];
        const config = widget.config.data[field];
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
              props
            );
          default:
            throw new Error(
              "Data source not supported: " + JSON.stringify(cfg.loader)
            );
        }
      });
    };

    const getFieldDataStore = (widgetId: string, field: string) =>
      once(`${widgetId}/${field}`, () => {
        const config = untrack(() => getFieldConfig(widgetId, field));
        if (!config) {
          return [() => {}];
        }
        let signal: any;
        switch (config.source) {
          case "userinput":
            let configMemo = getFieldConfigMemo(widgetId, field);
            return [() => configMemo().config?.value, () => {}];
          case "config": {
            let configMemo = getFieldConfigMemo(widgetId, field);
            return [() => configMemo().config, () => {}];
          }
          case "transient":
            return createSignal(config.config?.value);
          case "template":
          case "dynamic": {
            const propsGetterGenerator = getPropsGetterGenerator(
              widgetId,
              field
            );
            const propsGetter = createMemo(() => {
              const getter = propsGetterGenerator();
              if (getter) {
                return getter(propsGeneratorContext);
              }
              return {};
            });
            signal = getDyanmicDataResource(widgetId, field, propsGetter);
            /**
             * Note(sagar): manually trigger refetch so that we can control whether to
             * auto-refetch data on config change
             */
            const track = createReaction(() => {
              startTransition(() => signal[1].refetch());
              track(() => getFieldConfig(widgetId, field));
            });
            track(() => getFieldConfig(widgetId, field));
            return signal;
          }
          default:
            // @ts-expect-error
            throw new Error("Unsupported data source:" + config.source);
        }
      });

    Object.assign(editor.context, {
      useWidgetData(widgetId: string, field: string) {
        return getFieldDataStore(widgetId, field)[0];
      },
      useWidgetDataSetter(widgetId: string, field: string) {
        return untrack(() => {
          const source = getFieldConfig(widgetId, field)?.source;
          if (source == "transient") {
            const store = getFieldDataStore(widgetId, field);
            return store[1];
          } else if (source == "config") {
            return (value: any) => {
              untrack(() => {
                const fieldConfig = getFieldConfig(widgetId, field)!;
                if (!isEqual(fieldConfig.config, value)) {
                  ctx.updateWidget(
                    widgetId,
                    "config",
                    "data",
                    field,
                    "config",
                    value
                  );
                }
              });
            };
          }
          return () => {};
        });
      },
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
  widget: Widget,
  field: string,
  props: any
) {
  const { routes } = useApiContext();
  return await routes.queryWidgetData({
    appId,
    widgetId: widget.id,
    field,
    updatedAt: widget.updatedAt,
    props,
  });
}

export { withWidgetDataLoaders };
export type { WidgetDataContext };
