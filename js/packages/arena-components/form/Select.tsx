import { Select as K } from "@kobalte/core";
import CheckIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/tick";
import CaretSortIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/double-caret-vertical";
import { InlineIcon } from "../InlineIcon";
import { splitProps, useContext } from "solid-js";
import { ElementProps } from "./types";
import { StateContext } from "./state";

type SelectProps = {
  options: any[];
  placeholder?: string;
  itemClass?: string;
  contentClass?: string;
} & ElementProps;

export default function Select(props: SelectProps) {
  const { setState } = useContext(StateContext)!;
  // set initial value
  setState(props.name, props.value);

  const [attrs, _] = splitProps(props, [
    "value",
    "options",
    "placeholder",
    "class",
  ]);
  return (
    <>
      <K.Root
        {...attrs}
        onChange={(value) => {
          props.onChange?.(value);
          setState(props.name, value);
        }}
        itemComponent={(itemProps: any) => (
          <K.Item
            item={itemProps.item}
            class="select-item relative rounded flex items-center justify-between px-2 select-none outline-none data-[highlighted]:(bg-brand-10,text-white,outline-none)"
            classList={{
              [props.itemClass!]: Boolean(props.itemClass),
            }}
          >
            <K.ItemLabel>{itemProps.item.rawValue}</K.ItemLabel>
            <K.ItemIndicator class="select-item-indicator w-5 inline-flex items-center justify-center data-[disabled]:(text-accent-4,opacity-50,pointer-events-none)">
              <InlineIcon size="12px">
                <path d={CheckIcon[0]} />
              </InlineIcon>
            </K.ItemIndicator>
          </K.Item>
        )}
      >
        <K.Trigger
          class="select-trigger inline-flex items-center justify-between pr-2 pl-3 rounded-md outline-none bg-white border border-gray-100 hover:border-gray-300 focus-visible:ring-1"
          classList={{
            [props.class!]: Boolean(props.class),
          }}
        >
          <K.Value<string> class="select-value py-1 flex flex-grow gap-2 text-ellipsis whitespace-nowrap overflow-hidden data-[placeholder-shown]:text-accent-10 after:content-['*'] after:w-0 after:overflow-hidden">
            {(state) => state.selectedOption()}
          </K.Value>
          <K.Icon class="select-icon py-1 flex flex-grow-0 flex-shrink-0 basis-5 text-accent-11/90 justify-center">
            <InlineIcon size="10px">
              <path d={CaretSortIcon[0]} />
            </InlineIcon>
          </K.Icon>
        </K.Trigger>
        <K.Portal>
          <K.Content
            class="select-content bg-white rounded-md shadow-md"
            classList={{
              [props.contentClass!]: Boolean(props.contentClass),
            }}
          >
            <K.Listbox class="select-listbox overflow-y-auto max-h-96 p-2 focus:outline-none" />
          </K.Content>
        </K.Portal>
      </K.Root>
    </>
  );
}
