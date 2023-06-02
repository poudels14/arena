import { useEditorContext, TemplateStoreContext } from "../editor";
import { For, createMemo, createSignal } from "solid-js";
import { Form, Input } from "@arena/components/form";
// @ts-ignore
import debounce from "debounce";

const StyleEditor = () => {
  const { getSelectedWidgets } = useEditorContext();
  const { useWidgetById, updateWidget } =
    useEditorContext<TemplateStoreContext>();

  const classes = createMemo(() => {
    const activeWidgets = getSelectedWidgets();
    const widgetId = activeWidgets[0];
    const { config } = useWidgetById(widgetId)();
    if (config.class) {
      return config.class.split(" ");
    }
    return [];
  });

  const [inputValue, setInputValue] = createSignal();

  const deleteClass = debounce((c: string) => {
    const activeWidgets = getSelectedWidgets();
    const widgetId = activeWidgets[0];
    updateWidget(
      widgetId,
      "config",
      "class",
      classes()
        .filter((cl) => cl != c)
        .join(" ")
    );
  });

  const addClass = debounce((clazz: string) => {
    const activeWidgets = getSelectedWidgets();
    const widgetId = activeWidgets[0];
    const set = new Set(classes());
    clazz
      .split(" ")
      .map((c) => c.trim())
      .filter((c) => c.length > 0)
      .forEach((c) => set.add(c));
    updateWidget(widgetId, "config", "class", [...set].join(" "));
  });

  return (
    <div class="flex flex-col w-full h-full px-4">
      <div class="text-sm text-accent-9 pb-5">
        Use Tailwind class to style the widget. The class names should be
        separated by space.
      </div>
      <div class="flex-1 flex flex-col justify-end">
        <div class="flex flex-row flex-wrap gap-3 text-accent-4">
          <For each={classes()}>
            {(c) => (
              <div class="flex flex-row text-xs border border-accent-9 rounded-xl">
                <div class="px-3 py-0.5">{c}</div>
                <div
                  class="pl-1.5 pr-2 pt-0.5 pb-1 text-xs text-center rounded-r-xl overflow-hidden border-l border-accent-9 bg-accent-9 cursor-pointer"
                  onClick={() => deleteClass(c)}
                >
                  x
                </div>
              </div>
            )}
          </For>
        </div>
      </div>
      <div class="pt-4">
        <Form
          class="text-accent-11"
          onChange={(v) => setInputValue(v.class)}
          onSubmit={(value) => {
            addClass(value.class);
            setInputValue("");
          }}
        >
          <Input
            name="class"
            placeholder="Tailwind class"
            class="w-full py-1 text-sm"
            value={inputValue()}
          />
        </Form>
      </div>
    </div>
  );
};

export { StyleEditor };
