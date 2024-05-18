import { useEffect, useMemo } from "react";
import { useForm } from "@tanstack/react-form";
import TextareaAutosize from "react-textarea-autosize";
import * as Switch from "@radix-ui/react-switch";
import clsx from "clsx";

import { useEditorContext } from "../Context";

const NodeEditor = (props: { node: any }) => {
  const editor = useEditorContext();
  const agentNode = useMemo(() => {
    return editor.agentNodes.find((n) => n.id == props.node.data.type);
  }, [editor.agentNodes, props.node.type]);

  const form = useForm({
    defaultValues: props.node.data.config,
  });
  useEffect(() => {
    form.reset();
  }, [props.node]);

  const state = form.useStore((state) => state.values);
  useEffect(() => {
    editor.setNodes((prev) => {
      const nodes = [...prev];
      const node = nodes.find((n) => n.id == props.node.id)!;
      node.data.config = state;
      return nodes;
    });
  }, [state]);

  return (
    <div className="w-full h-full">
      <div className="title py-1 font-semibold text-base text-center border-b border-gray-300">
        {props.node.data.label}
      </div>
      {agentNode?.config?.length! > 0 && (
        <div className="px-2 space-y-1">
          <div className="py-3 text-base font-bold text-gray-700">Config</div>
          <div className="space-y-3">
            {agentNode?.config.map((metadata, index) => {
              return (
                <form.Field
                  key={index}
                  name={metadata.id}
                  children={(field) => (
                    <ConfigFieldEditor
                      key={index}
                      metadata={metadata}
                      value={field.state.value}
                      field={field}
                    />
                  )}
                />
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};

const ConfigFieldEditor = (props: {
  metadata: any;
  value: any;
  field: any;
}) => {
  const { schema, ui } = props.metadata;
  return (
    <div
      className={clsx("space-y-1", {
        "flex justify-between": schema.type == "boolean",
      })}
    >
      <div className="py-2 text-sm font-semibold text-gray-800">
        {props.metadata.label}
      </div>
      <div className="text-sm text-gray-600 *:w-full *:outline-none *:rounded focus:*:ring-1 *:ring-indigo-300 *:bg-slate-50">
        {schema.type == "string" && ui.type == "textarea" && (
          <TextareaAutosize
            className="px-3 py-1.5"
            minRows={3}
            value={props.value}
            onChange={(e) => props.field.handleChange(e.target.value)}
          />
        )}
        {schema.type == "string" && ui.type != "textarea" && (
          <input
            className="px-3 py-1.5"
            value={props.value}
            onChange={(e) => props.field.handleChange(e.target.value)}
          />
        )}
        {schema.type == "number" && (
          <input
            className="px-3 py-1.5"
            type="number"
            value={props.value}
            onChange={(e) => props.field.handleChange(e.target.value)}
          />
        )}
        {schema.type == "boolean" && (
          <Switch.Root
            className="relative !w-[38px] h-[21px] bg-gray-300 !rounded-full shadow data-[state=checked]:bg-indigo-400"
            id={props.metadata.id}
          >
            <Switch.Thumb className="block !w-[18px] h-[18px] bg-white !rounded-full shadow translate-x-[2px] will-change-transform data-[state=checked]:translate-x-[19px]" />
          </Switch.Root>
        )}
      </div>
    </div>
  );
};

export { NodeEditor };
