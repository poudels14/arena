import { useMemo, useRef } from "react";
import { Position } from "reactflow";
import clsx from "clsx";

import { useEditorContext } from "../Context";
import { NodeHandle } from "./Handle";

const AgentNode = (props: any) => {
  const { selectedNodes, agentNodes } = useEditorContext();
  const isHighlighted = useMemo(
    () => selectedNodes.find((n) => n == props.id),
    [selectedNodes]
  );

  const nodeConfig = agentNodes.find((node) => node.id == props.data.type) || {
    icon: undefined,
    config: [],
    inputs: [],
    outputs: [],
  };

  let ref = useRef<HTMLDivElement>(null);
  const nodeCoords = useMemo(() => {
    if (!ref.current) {
      return { left: 0, top: 0 };
    }
    return ref.current?.getBoundingClientRect();
  }, [ref.current]);

  return (
    <div
      ref={ref}
      className={clsx(
        "node w-72 bg-gray-50 border border-gray-300 rounded overflow-hidden",
        isHighlighted && "border-indigo-400 shadow-md shadow-indigo-200"
      )}
    >
      <div>
        <div
          className={clsx(
            "title flex p-3 items-center space-x-3 border-b border-gray-200 overflow-hidden text-sm text-ellipsis whitespace-nowrap",
            isHighlighted && "bg-slate-100"
          )}
        >
          {nodeConfig.icon && (
            <div
              className={clsx({
                "text-gray-500": !isHighlighted,
                "text-indigo-500": isHighlighted,
              })}
              dangerouslySetInnerHTML={{ __html: nodeConfig.icon }}
            ></div>
          )}
          <div className="overflow-hidden text-ellipsis">
            {props.data.label}
          </div>
        </div>
        {nodeConfig.inputs.length > 0 && (
          <div>
            <div className="py-1 text-center bg-gray-100 font-semibold">
              Input
            </div>
            <div className="text-xs px-2 py-2 space-y-1.5">
              {nodeConfig.inputs.map((input: any, index: number) => {
                return (
                  <NodeHandle
                    key={index}
                    nodeId={props.id}
                    nodeCoords={nodeCoords}
                    type="target"
                    position={Position.Left}
                    id={input.id}
                    label={input.label}
                  />
                );
              })}
            </div>
          </div>
        )}
        {nodeConfig.outputs.length > 0 && (
          <div>
            <div className="py-1 text-center bg-gray-100 font-semibold">
              Output
            </div>
            <div className="text-xs px-2 py-2 space-y-1.5 text-right">
              {nodeConfig.outputs.map((output: any, index: number) => {
                return (
                  <NodeHandle
                    key={index}
                    nodeId={props.id}
                    nodeCoords={nodeCoords}
                    type="source"
                    position={Position.Right}
                    id={output.id}
                    label={output.label}
                  />
                );
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export { AgentNode };
