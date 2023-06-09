import { useEditorContext, TemplateStoreContext } from "../editor";
import {
  For,
  Match,
  Show,
  Switch,
  createComputed,
  createMemo,
  createSelector,
} from "solid-js";
import type { DataSource } from "@arena/widgets";
import { createStore } from "@arena/solid-store";
import { CodeEditor } from "@arena/components";
// @ts-ignore
import debounce from "debounce";
import { Form, Select } from "@arena/components/form";

const Data = () => {
  const { getSelectedWidgets } = useEditorContext();
  const { useTemplate, useWidgetById } =
    useEditorContext<TemplateStoreContext>();

  const [state, setState] = createStore<{ selectedField: string | null }>({
    selectedField: null,
  });

  const fieldMetadata = createMemo(() => {
    const activeWidgets = getSelectedWidgets();
    const widgetId = activeWidgets[0];
    const widget = useWidgetById(widgetId)();
    const template = useTemplate(widget.template.id);

    return Object.entries(widget.config.data)
      .filter(([_, config]) => ["dynamic", "userinput"].includes(config.source))
      .map(([fieldName, fieldConfig]) => {
        const templateConfig = template.metadata.data[fieldName];
        return {
          widgetId,
          fieldName,
          title: templateConfig?.title || "Unrecognized field",
          fieldConfig: fieldConfig as DataSource.Dynamic,
        };
      });
  });

  const selectedFieldMetadata = createMemo(() => {
    const field = state.selectedField();
    const metadata = fieldMetadata();
    return metadata.find((c) => c.fieldName == field) || metadata[0];
  });

  const isSelected = createSelector(
    () => state.selectedField() || fieldMetadata()[0]?.fieldName
  );

  return (
    <div class="flex flex-row px-2 h-full space-x-2 text-brand-1">
      <Switch>
        <Match when={fieldMetadata().length == 0}>
          <div class="flex flex-col w-full text-center justify-center text-brand-8">
            There's no configurable data for this widget
          </div>
        </Match>
        <Match when={true}>
          <div class="w-28 space-y-1">
            <div class="text-xs">Data Fields</div>
            <div class="space-y-1">
              <For each={fieldMetadata()}>
                {(field) => {
                  return (
                    <Field
                      {...field}
                      selected={isSelected(field.fieldName)}
                      setSelectedField={(name) =>
                        setState("selectedField", name)
                      }
                    />
                  );
                }}
              </For>
            </div>
          </div>
          <Show when={selectedFieldMetadata()}>
            <FieldEditor metadata={selectedFieldMetadata()!} />
          </Show>
        </Match>
      </Switch>
    </div>
  );
};

type FieldMetadata = {
  widgetId: string;
  fieldName: string;
  title: string;
  fieldConfig: DataSource.Dynamic;
};

const Field = (
  props: FieldMetadata & {
    selected: boolean;
    setSelectedField: (name: string) => void;
  }
) => {
  return (
    <div
      class="px-2 py-1 text-xs cursor-pointer rounded hover:bg-brand-12/40"
      classList={{
        "bg-brand-12/70": props.selected,
      }}
      onClick={() => props.setSelectedField(props.fieldName)}
    >
      {props.title}
    </div>
  );
};

const FieldEditor = (props: { metadata: FieldMetadata }) => {
  const { updateWidget, getAvailableResources } = useEditorContext();
  const [state, setState] = createStore({
    loader: null as FieldMetadata["fieldConfig"]["config"]["loader"] | null,
    resource: null,
  });
  createComputed(() => {
    setState({
      loader: props.metadata.fieldConfig.config.loader,
    });
  });

  const updateDataLoaderConfig = (
    config: Omit<FieldMetadata["fieldConfig"]["config"], "value">
  ) => {
    const { widgetId, fieldName } = props.metadata;
    updateWidget(widgetId, "config", "data", fieldName, "config", config);
  };
  return (
    <Show when={props.metadata}>
      <div class="flex-1 px-2 py-4 space-y-2 overflow-y-auto no-scrollbar">
        <div class="flex flex-row space-x-2">
          <div>Data Source</div>
          <Form class="text-brand-12 space-y-2">
            <Select
              name="loader"
              placeholder="Select data source"
              class="w-60 px-10 text-xs text-brand-12 rounded"
              contentClass="z-[999999]"
              itemClass="text-xs"
              value={state.loader()}
              options={[
                { id: "@client/json", label: "Inline Data" },
                { id: "@client/js", label: "Client Javascript" },
                {
                  id: "@arena/server-function",
                  label: "Custom Server Function",
                },
                { id: "@arena/sql/postgres", label: "Postgres Database" },
              ]}
              optionTextValue="label"
              optionValue="id"
              onChange={(loader) => {
                if (loader != "@arena/sql/postgres") {
                  updateDataLoaderConfig({ loader });
                }
                setState({
                  loader,
                });
              }}
            />
            <Show when={state.loader() == "@arena/sql/postgres"}>
              <Select
                name="db"
                placeholder="Select Postgress database"
                class="w-60 px-10 text-xs text-brand-12 rounded"
                contentClass="z-[999999]"
                itemClass="text-xs"
                // @ts-expect-error
                value={props.metadata.fieldConfig.config.db}
                options={getAvailableResources()}
                optionTextValue="name"
                optionValue="id"
                onChange={(resource: string) => {
                  updateDataLoaderConfig({
                    loader: state.loader()!,
                    // @ts-expect-error
                    db: resource,
                  });
                  setState((prev: any) => {
                    return {
                      ...prev,
                      resource,
                    };
                  });
                }}
              />
            </Show>
          </Form>
        </div>
        <div>
          <DataSourceEditor metadata={props.metadata} />
        </div>
      </div>
    </Show>
  );
};

const DataSourceEditor = (props: { metadata: FieldMetadata }) => {
  const { updateWidget } = useEditorContext();
  const editorProps = createMemo(() => {
    const { config } = props.metadata.fieldConfig;
    const isJavascript =
      config.loader == "@arena/server-function" ||
      config.loader == "@client/js";
    const isSql = config.loader == "@arena/sql/postgres";
    return {
      code:
        config.loader == "@client/json" && typeof config.value != "string"
          ? JSON.stringify(config.value, null, 2)
          : config.value,
      lang: isJavascript ? "javascript" : isSql ? "sql" : "text",
    } as { lang: "sql"; code: string };
  });

  const onChange = debounce((value: string) => {
    const { widgetId, fieldName, fieldConfig } = props.metadata;
    if (editorProps().code == value) {
      // early return if value didnt change
      return;
    }
    const config = fieldConfig.config;
    updateWidget(widgetId, "config", "data", fieldName, "config", {
      ...config,
      value:
        config.loader == "@client/json" && typeof config.value != "string"
          ? JSON.parse(value)
          : value,
    });
  }, 300);

  return (
    <div class="w-full py-2 bg-brand-1 text-black">
      <CodeEditor
        lang={editorProps().lang}
        value={editorProps().code}
        onChange={onChange}
      />
    </div>
  );
};

export default Data;
