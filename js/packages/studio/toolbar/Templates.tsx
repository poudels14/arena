import { Template } from "@arena/widgets";
import { useEditorContext, TemplateStoreContext } from "../editor";
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
      <div class="flex flex-col px-2 w-72 h-full space-y-1 text-brand-1 border-r border-brand-11/30">
        <div class="border-b border-brand-11/40">
          <input
            type="text"
            class="px-2 w-36 text-xs bg-transparent outline-none"
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
      <div class=""></div>
    </div>
  );
};

const TemplateCard = (props: { metadata: Template.Metadata<any> }) => {
  const draggable = createDraggable("template-card-" + props.metadata.id, {
    templateId: props.metadata.id,
  });

  return (
    <div
      class="px-2 py-2 cursor-pointer rounded bg-brand-12/40 hover:bg-brand-12/80"
      ref={draggable.ref}
    >
      {props.metadata.name}
    </div>
  );
};

export { Templates };
