import { Accessor, createMemo } from "solid-js";
import { Plugin } from "./types";
import { DataSource, DataSources } from "../widget/types/data";

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
          default:
            throw new Error("Data source not supported: " + cfg.source);
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

function useInlineDataSource<T>(config: DataSources.InlineSourceConfig<T>) {
  return config.value;
}

export { withWidgetDataLoaders };
export type { WidgetDataContext };
