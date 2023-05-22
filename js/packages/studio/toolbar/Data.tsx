import { useEditorContext, TemplateStoreContext } from "@arena/appkit/editor";
import { For, Match, Show, Switch, createMemo, createSignal } from "solid-js";
import type { DataSource } from "@arena/appkit/widget";
import { createStore } from "@arena/solid-store";
import { CodeEditor } from "@arena/components";
import debounce from "debounce";

const Data = () => {
  const { getSelectedWidgets } = useEditorContext();
  const { useTemplate, useWidgetById } =
    useEditorContext<TemplateStoreContext>();

  const [state, setState] = createStore<{ selectedField: string | null }>({
    selectedField: null,
  });

  const fieldConfigs = createMemo(() => {
    const activeWidgets = getSelectedWidgets();
    const widget = useWidgetById(activeWidgets[0])();
    const template = useTemplate(widget.template.id);

    return Object.entries(widget.config.data)
      .filter(([_, config]) => "dynamic" == config.type)
      .map(([fieldName, fieldConfig]) => {
        const templateConfig = template.metadata.data[fieldName];
        return {
          widgetId: widget.id,
          name: fieldName,
          title: templateConfig?.title || "Unrecognized field",
          dataSource: fieldConfig.config as DataSource.Dynamic<any>["config"],
        };
      });
  });

  const selectedFieldConfig = createMemo(() => {
    const field = state.selectedField();
    const configs = fieldConfigs();
    return configs.find((c) => c.name == field) || configs[0];
  });

  return (
    <div class="flex flex-row px-2 h-full space-x-2 text-white">
      <Switch>
        <Match when={fieldConfigs().length == 0}>
          <div class="flex flex-col w-full text-center justify-center text-slate-500">
            There's no configurable data for this widget
          </div>
        </Match>
        <Match when={true}>
          <div class="w-40 space-y-1">
            <div class="text-xs">Data Fields</div>
            <div>
              <For each={fieldConfigs()}>
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
          <Show when={selectedFieldConfig()}>
            <FieldEditor config={selectedFieldConfig()!} />
          </Show>
        </Match>
      </Switch>
    </div>
  );
};

type FieldConfig = {
  widgetId: string;
  name: string;
  title: string;
  dataSource: DataSource.Dynamic<any>["config"];
};

const Field = (
  props: FieldConfig & { setSelectedField: (name: string) => void }
) => {
  return (
    <div
      class="px-2 py-1 text-xs cursor-pointer rounded bg-slate-600 hover:bg-slate-500"
      onClick={() => props.setSelectedField(props.name)}
    >
      {props.title}
    </div>
  );
};

const FieldEditor = (props: { config: FieldConfig }) => {
  const [value, setValue] = createSignal(props.config.dataSource.source);
  return (
    <Show when={props.config}>
      <div class="flex-1 px-2 py-4 space-y-2 overflow-y-auto no-scrollbar">
        <div class="flex flex-row space-x-2">
          <div>Data Source</div>
          <select
            class="px-2 text-sm text-black rounded-sm outline-none appearance-none after:content-['*'] after:(w-4,h-2,bg-gray-400,clip-path-[polygon(100%-0%,0-0%,50%-100%)])"
            value={props.config.dataSource.source}
            onChange={(e) => setValue(e.target.value as any)}
          >
            <For
              each={[
                ["inline", "Inline Data"],
                ["client/js", "Client Javascript"],
                ["server/sql", "SQL Query"],
                ["server/js", "Javascript Server Function"],
              ]}
            >
              {(source) => <option value={source[0]}>{source[1]}</option>}
            </For>
          </select>
        </div>
        <div>
          <DataSourceEditor config={props.config.dataSource} />
        </div>
      </div>
    </Show>
  );
};

const DataSourceEditor = (props: {
  config: DataSource.Dynamic<any>["config"];
}) => {
  const editorProps = createMemo(() => {
    const config = props.config as any;
    return {
      code: config.query ?? JSON.stringify(config.value, null, 2),
      lang: config.source == "server/sql" ? "sql" : "javascript",
    } as { lang: "sql"; code: string };
  });

  const onChange = debounce((value: string) => {
    console.log("VALUE =", value);
  }, 300);

  return (
    <div class="w-full py-2 bg-gray-100 text-black">
      <CodeEditor
        lang={editorProps().lang}
        value={editorProps().code}
        onChange={onChange}
      />
    </div>
  );
};

export { Data };
