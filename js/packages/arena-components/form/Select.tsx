import { Select as K } from "@kobalte/core";
import CheckIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/tick";
import CaretSortIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/double-caret-vertical";
import { InlineIcon } from "../InlineIcon";
import { splitProps, useContext } from "solid-js";
import { ElementProps } from "./types";
import { useStateContext } from "./state";

type StringOrKeyOf<T> = T extends string ? undefined : keyof T;

type SelectProps<T> = {
  options: T[];
  /**
   * Field name of `options` prop that should be used as form value
   */
  optionValue?: StringOrKeyOf<T>;
  /**
   * Field name of `options` prop that should be used to display in dropdown
   */
  optionTextValue?: StringOrKeyOf<T>;
  /**
   * Field name of `options` prop that should is used to check whether the
   * item is disabled. For example,
   * if `options = [ { name: "id", title: "Id", disabled: false }]`,
   * `optionDisabled = "disabled"` should be passed
   */
  optionDisabled?: StringOrKeyOf<T>;
  placeholder?: string;
  triggerClass?: string;
  itemClass?: string;
  contentClass?: string;
} & ElementProps;

export default function Select<T>(props: SelectProps<T>) {
  const getValue = (option: T) => {
    // @ts-expect-error
    return typeof option === "object" ? option[props.optionValue!] : option;
  };

  const getLabel = (option: T) => {
    return (
      typeof option === "object" ? option?.[props.optionTextValue!] : option
    ) as string;
  };

  const { state, setState } = useStateContext<any>();
  // set initial value
  setState(props.name, props.value);

  const [_, rest] = splitProps(props, [
    "name",
    "value",
    "onChange",
    "triggerClass",
    "itemClass",
    "contentClass",
  ]);
  return (
    <K.Root
      {...rest}
      // @ts-expect-error
      value={props.options.find((o) => getValue(o) == state[props.name]?.())}
      onChange={(option) => {
        let v = getValue(option);
        props.onChange?.(v);
        setState(props.name, v);
      }}
      itemComponent={(itemProps: any) => (
        <K.Item
          item={itemProps.item}
          class="select-item relative rounded-sm flex items-center justify-between px-2 py-1 select-none outline-none data-[highlighted]:(bg-brand-10,text-white,outline-none)"
          classList={{
            [props.itemClass!]: Boolean(props.itemClass),
          }}
        >
          <K.ItemLabel>{getLabel(itemProps.item.rawValue)}</K.ItemLabel>
          <K.ItemIndicator class="select-item-indicator w-5 inline-flex items-center justify-center data-[disabled]:(text-accent-4,opacity-50,pointer-events-none)">
            <InlineIcon size="12px">
              <path d={CheckIcon[0]} />
            </InlineIcon>
          </K.ItemIndicator>
        </K.Item>
      )}
    >
      <K.Trigger
        class="select-trigger inline-flex items-center justify-between pr-2 pl-3 rounded-md outline-none bg-white border border-gray-100 hover:border-gray-300 focus-visible:ring-1 ring-inset focus:ring-1"
        classList={{
          [props.triggerClass!]: Boolean(props.triggerClass),
        }}
      >
        <K.Value<T> class="select-value py-1 flex flex-grow gap-2 text-ellipsis whitespace-nowrap overflow-hidden data-[placeholder-shown]:text-accent-9 after:content-['*'] after:w-0 after:overflow-hidden">
          {(state) => getLabel(state.selectedOption())}
        </K.Value>
        <K.Icon class="select-icon py-1 flex flex-grow-0 flex-shrink-0 basis-5 text-accent-11/90 justify-center">
          <InlineIcon size="10px">
            <path d={CaretSortIcon[0]} />
          </InlineIcon>
        </K.Icon>
      </K.Trigger>
      <K.Portal>
        <K.Content
          class="select-content bg-white rounded-md shadow-xl border border-accent-8"
          classList={{
            [props.contentClass!]: Boolean(props.contentClass),
          }}
        >
          <K.Listbox class="select-listbox overflow-y-auto max-h-96 p-1 focus:outline-none" />
        </K.Content>
      </K.Portal>
    </K.Root>
  );
}
