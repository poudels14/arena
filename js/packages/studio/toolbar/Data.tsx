import { useEditorContext, TemplateStoreContext } from "@arena/appkit/editor";
import { For, Match, Switch, createMemo } from "solid-js";
import { useAppContext } from "@arena/appkit";

const Data = () => {
  const { getSelectedWidgets } = useAppContext();

  const { useTemplate, useWidgetById } =
    useEditorContext<TemplateStoreContext>();
  const fieldConfigs = createMemo(() => {
    const activeWidgets = getSelectedWidgets();
    const widget = useWidgetById(activeWidgets[0].id!)();
    const template = useTemplate(widget.template.id);

    return Object.entries(template.metadata.data)
      .filter(([_, config]) => ["dynamic"].includes(config.dataSource.type))
      .map(([fieldName, templateconfig]) => {
        return {
          name: fieldName,
          title: templateconfig.title,
          dataSource: templateconfig.dataSource,
          config: widget.config.data[fieldName],
        };
      });
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
          <div class="w-40">
            <For each={fieldConfigs()}>
              {(field) => {
                return <Field title={field.title} />;
              }}
            </For>
          </div>
          <div class="">Data field editor</div>
        </Match>
      </Switch>
    </div>
  );
};

const Field = (props: { title: string }) => {
  return (
    <div class="px-2 py-1 cursor-pointer rounded bg-slate-600 hover:bg-slate-500">
      {props.title}
    </div>
  );
};

export { Data };
