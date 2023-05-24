import { Accessor, createMemo } from "solid-js";
import { Plugin } from "./types";
import { DataSource } from "@arena/widgets/schema/data";

type WidgetDataContext = {
  useWidgetData: <T>(
    widgetId: string,
    field: string,
    configAccessor: Accessor<DataSource<T>>
  ) => Accessor<T>;
};

function useWidgetData<T>(
  widgetId: string,
  field: string,
  configAccessor: Accessor<DataSource<T>>
) {
  const fieldData = createMemo(() => {
    const config = configAccessor();
    switch (config.type) {
      case "template":
      case "dynamic": {
        const cfg = config.config;
        switch (cfg.source) {
          case "inline":
            return useInlineDataSource(cfg);
          case "client/js":
            return useClientJsDataSource(cfg);
          case "server/js":
          case "server/sql":
            return useServerFunctionDataSource(cfg);
          default:
            throw new Error(
              "Data source not supported: " + JSON.stringify(cfg)
            );
        }
      }
      default:
        throw new Error("Data source not supported: " + config.type);
    }
  });

  // TODO(sagar): cache fieldData accessor such that when other widgets
  // access data for a widget, the accessor can be returned
  //   - Can we trigger suspense when a widget access another widget's
  //     data but that widget hasn't be initialized yet or is ready?
  //   -
  return fieldData;
}

const withWidgetDataLoaders: Plugin<{}, {}, {}> = (config) => (editor) => {
  Object.assign(editor.context, {
    useWidgetData,
  });
};

function useInlineDataSource<T>(config: DataSource.InlineSourceConfig<T>) {
  return config.value;
}

function useClientJsDataSource(config: DataSource.ClientJsConfig) {
  // TODO(sagar): load widget data
  return [];
}

function useServerFunctionDataSource(
  config: DataSource.ServerSqlConfig | DataSource.ServerJsConfig
) {
  // TODO(sagar): load widget data
  return [];
}

export { withWidgetDataLoaders };
export type { WidgetDataContext };
