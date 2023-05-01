import { Template } from "@arena/appkit/widget";
import { useEditorContext, TemplateStoreContext } from "@arena/appkit/editor";
import { For, createMemo } from "solid-js";
import { createDraggable } from "@arena/solid-dnd";

const Templates = () => {
  const { useTemplates } = useEditorContext<TemplateStoreContext>();
  const getTemplates = useTemplates();

  const templateList = createMemo(() => {
    const templates = getTemplates();
    return Object.values(templates).sort((a, b) =>
      a.metadata.name.localeCompare(b.metadata.name)
    );
  });

  return (
    <div class="flex flex-row h-full">
      <div class="flex flex-col px-2 w-72 h-full space-y-1 text-white border-r border-slate-600">
        <div class="border-b border-slate-600">
          <input
            type="text"
            class="px-2 w-36 text-xs bg-transparent outline-none border-slate-500"
            placeholder="Search template"
          />
        </div>
        <div class="pt-1 space-y-1 text-sm overflow-y-auto no-scrollbar select-none">
          <For each={templateList()}>
            {(template) => {
              return <TemplateCard metadata={template.metadata} />;
            }}
          </For>
        </div>
      </div>
      <div class="">TEMPLATE PREVIEW</div>
    </div>
  );
};

const TemplateCard = (props: { metadata: Template.Metadata<any> }) => {
  const draggable = createDraggable("template-card-" + props.metadata.id, {
    templateId: props.metadata.id,
  });

  void draggable;
  return (
    <div
      class="px-2 py-2 cursor-pointer rounded bg-slate-600 hover:bg-slate-500"
      use:draggable={draggable}
    >
      {props.metadata.name}
    </div>
  );
};

export { Templates };
