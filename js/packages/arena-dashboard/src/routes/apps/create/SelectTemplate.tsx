import { For } from "solid-js";
import { BUILTIN_APPS } from "~/BUILTIN_APPS";

const SelectTemplate = (props: any) => {
  const tempaltes = BUILTIN_APPS;

  return (
    <div class="pb-5 space-y-3">
      <div>Select a template</div>
      <div>
        <For each={tempaltes}>
          {(template) => {
            return (
              <div
                class="relative py-2 w-60 h-36 rounded-md cursor-pointer bg-brand-3 hover:bg-brand-5"
                onClick={() => {
                  props.next(template);
                }}
              >
                <div class="absolute bottom-0 p-2 w-full text-center font-medium text-brand-12">
                  {template.name}
                </div>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
};

export default SelectTemplate;
