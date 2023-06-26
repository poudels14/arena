import { createMemo } from "solid-js";
import { Select } from "@arena/components/form";
import type { Template } from "..";
import { filterTextClasses } from "@arena/uikit/classes";

const metadata: Template.Metadata<{
  options: any[];
  placeholder: string;
  value: string;
}> = {
  id: "at-select",
  name: "Select",
  description: "Selection dropdown",
  data: {
    placeholder: {
      title: "Placeholder",
      source: "userinput",
      default: {
        value: "Select",
      },
      preview: "Select",
    },
    options: {
      title: "Options",
      source: "userinput",
      default: {
        value: ["Option 1", "Option 2", "Option 3"],
      },
      preview: ["Option 1", "Option 2", "Option 3"],
    },
    value: {
      title: "Value",
      source: "config",
    },
  },
  class: "w-60 text-sm text-brand-12",
};

const SelectWidget = (
  props: Template.Props<{
    placeholder: string;
    options: any[];
  }>
) => {
  const itemClass = createMemo(() => filterTextClasses(props.attrs.classList));
  return (
    <Select
      name="name"
      placeholder={props.placeholder}
      options={props.options || []}
      triggerClass="w-full"
      itemClass={itemClass()}
      contentClass="z-[999999]"
      {...props.attrs}
    />
  );
};

export default SelectWidget;
export { metadata };
