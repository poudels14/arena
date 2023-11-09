import { For, Show } from "solid-js";
import { InlineIcon } from "@arena/components";

import AssistantsIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/chat";

const Assistants = (props: {
  isSelected: (id: string) => boolean;
  setSelected: (id: string) => void;
}) => {
  const isSelected = (id: string) => props.isSelected("/assistants/" + id);
  const setSelected = (id: string) => props.setSelected("/assistants/" + id);

  return (
    <div class="py-3 space-y-1 text-xs font-medium text-gray-600">
      <div class="px-2 py-1 table text-accent-12">
        <InlineIcon size="12px" class="table-cell">
          <path d={AssistantsIcon[0]} />
        </InlineIcon>
        <div class="table-cell pl-2 w-full">Assistants</div>
      </div>

      <div class="">
        <For
          each={[
            {
              id: "default",
              name: "Default",
            },
            {
              id: "assistant-1",
              name: "Assistant 1",
            },
            {
              id: "assistant-2",
              name: "Assistant 2",
            },
          ]}
        >
          {(assistant) => (
            <AssistantTab
              id={assistant.id}
              name={assistant.name}
              isSelected={isSelected}
              setSelected={setSelected}
            />
          )}
        </For>
      </div>
    </div>
  );
};

const AssistantTab = (props: {
  id: string;
  name: string;
  isSelected: (id: string) => boolean;
  setSelected: (id: string) => void;
}) => {
  const assistantSelected = () => {
    return props.isSelected(props.id) || props.isSelected(props.id + "/configure");
  };
  return (
    <div
      class="px-3 py-1 cursor-pointer"
      classList={{
        "hover:text-accent-12 hover:bg-gray-100 text-accent-11":
          !assistantSelected(),
      }}
      onClick={() => !assistantSelected() && props.setSelected(props.id)}
    >
      <div
        classList={{
          "font-bold text-accent-12": assistantSelected(),
        }}
      >
        {props.name}
      </div>
      <Show when={assistantSelected()}>
        <div class="py-1">
          <div
            class="py-1 px-2 hover:bg-gray-100"
            classList={{
              "font-bold bg-gray-100 text-accent-12": props.isSelected(
                props.id
              ),
            }}
            onClick={() => props.setSelected(props.id)}
          >
            Chat
          </div>
          <div
            class="py-1 px-2 hover:bg-gray-100"
            classList={{
              "font-bold bg-gray-100 text-accent-12": props.isSelected(
                props.id + "/configure"
              ),
            }}
            onClick={() => props.setSelected(props.id + "/configure")}
          >
            Configure
          </div>
        </div>
      </Show>
    </div>
  );
};

export { Assistants };
