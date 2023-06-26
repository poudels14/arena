import { createMemo } from "solid-js";
import { Select } from "@arena/components/form";
import type { Template } from "..";
import { filterTextClasses } from "@arena/uikit/classes";

const metadata: Template.Metadata<{
  resourceType: string;
  placeholder: string;
  value: string;
}> = {
  id: "@arena/resource-picker",
  name: "Resource Picker",
  description: "Select a resource from a list of linked resources",
  data: {
    resourceType: {
      title: "Resource Type",
      source: "userinput",
      editor: {
        type: "select",
        options: {
          source: "@arena/resource/type",
        },
      },
      default: {
        value: undefined,
      },
      preview: "Select a resource type",
    },
    placeholder: {
      title: "Placeholder",
      source: "userinput",
      default: {
        value: "Select a resource",
      },
      preview: "Select a resource",
    },
    value: {
      title: "Value",
      source: "config",
      default: null,
    },
  },
  class: "w-60 text-sm text-brand-12",
};

const ResourcePicker = (
  props: Template.Props<{
    resourceType: string | undefined;
    placeholder: string;
    value: string;
  }>
) => {
  const itemClass = createMemo(() => filterTextClasses(props.attrs.classList));
  const { useResources } = props.Editor.useContext();
  console.log("props.resourceType =", props.resourceType);
  return (
    <Select
      name="resource"
      placeholder={props.placeholder || "Select a resource"}
      options={
        useResources().filter(
          (r) => props.resourceType == undefined || r.type == props.resourceType
        ) || []
      }
      value={props.value}
      optionValue={"id"}
      optionTextValue={"name"}
      triggerClass="w-full"
      itemClass={itemClass()}
      contentClass="z-[999999]"
      {...props.attrs}
    />
  );
};

export default ResourcePicker;
export { metadata };
