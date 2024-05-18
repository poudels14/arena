import { useMemo, useRef } from "react";
import { Position } from "reactflow";
import clsx from "clsx";

import { useEditorContext } from "../Context";
import { NodeHandle } from "./Handle";

const AgentInput = (props: any) => {
  const { agentNodes } = useEditorContext();

  const nodeConfig = agentNodes.find((node) => node.id == props.data.type) || {
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
        "min-h-[theme(spacing.52)] bg-indigo-200 border border-gray-400 rounded overflow-hidden"
      )}
    >
      <div className="">
        <div
          className={clsx(
            "p-3 label border-b border-gray-100 bg-indigo-400 text-white overflow-hidden text-ellipsis whitespace-nowrap"
          )}
        >
          Agent Input
        </div>
        <div className={clsx("text-gray-600")}>
          {nodeConfig.outputs.length > 0 && (
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
          )}
        </div>
      </div>
    </div>
  );
};

export { AgentInput };
