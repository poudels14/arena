import { useEditorContext, TemplateStoreContext } from "../editor";
import { For, Match, Show, Switch, createMemo } from "solid-js";
import type { DataSource } from "@arena/widgets";
import { createStore } from "@arena/solid-store";
import { CodeEditor } from "@arena/components";
// @ts-ignore
import debounce from "debounce";

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
      .filter(([_, config]) => "dynamic" == config.source)
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

  return (
    <div class="flex flex-row px-2 h-full space-x-2 text-brand-1">
      <Switch>
        <Match when={fieldMetadata().length == 0}>
          <div class="flex flex-col w-full text-center justify-center text-brand-8">
            There's no configurable data for this widget
          </div>
        </Match>
        <Match when={true}>
          <div class="w-40 space-y-1">
            <div class="text-xs">Data Fields</div>
            <div>
              <For each={fieldMetadata()}>
                {(field) => {
                  return (
                    <Field
                      {...field}
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
  props: FieldMetadata & { setSelectedField: (name: string) => void }
) => {
  return (
    <div
      class="px-2 py-1 text-xs cursor-pointer rounded bg-brand-12/70 hover:bg-brand-12/40"
      onClick={() => props.setSelectedField(props.fieldName)}
    >
      {props.title}
    </div>
  );
};

const FieldEditor = (props: { metadata: FieldMetadata }) => {
  const { updateWidget } = useEditorContext();
  const setDataLoader = (
    loader: FieldMetadata["fieldConfig"]["config"]["loader"]
  ) => {
    const { widgetId, fieldName } = props.metadata;
    updateWidget(widgetId, "config", "data", fieldName, "config", {
      // @ts-expect-error
      loader,
      // TODO(sagar): set resource if selected
      // resource: "",
    });
  };
  return (
    <Show when={props.metadata}>
      <div class="flex-1 px-2 py-4 space-y-2 overflow-y-auto no-scrollbar">
        <div class="flex flex-row space-x-2">
          <div>Data Source</div>
          <select
            class="px-2 text-sm text-brand-12 rounded-sm outline-none appearance-none"
            value={props.metadata.fieldConfig.config.loader}
            onChange={(e) => setDataLoader(e.target.value as any)}
          >
            <For
              each={[
                ["@client/json", "Inline Data"],
                ["@client/js", "Client Javascript"],
                ["@arena/sql/postgres", "Postgres"],
                ["@arena/server-function", "Custom Server Function"],
              ]}
            >
              {(source) => <option value={source[0]}>{source[1]}</option>}
            </For>
          </select>
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
    return {
      code:
        config.loader == "@client/json" && typeof config.value != "string"
          ? JSON.stringify(config.value, null, 2)
          : config.value,
      lang:
        config.loader == "@arena/server-function" || "@client/js"
          ? "javascript"
          : config.loader == "@arena/sql/postgres"
          ? "sql"
          : "text",
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

export { Data };
