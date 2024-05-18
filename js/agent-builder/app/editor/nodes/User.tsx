import { useMemo, useRef } from "react";
import { Position } from "reactflow";
import clsx from "clsx";

import { useEditorContext } from "../Context";
import { NodeHandle } from "./Handle";

const User = (props: any) => {
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
        "w-[3000px] bg-red-200 border border-red-400 rounded overflow-hidden"
      )}
    >
      <div className="flex justify-center text-xs px-2 py-2 space-x-1.5">
        {nodeConfig.inputs.map((input: any, index: number) => {
          return (
            <NodeHandle
              key={index}
              nodeId={props.id}
              nodeCoords={nodeCoords}
              type="target"
              position={Position.Top}
              id={input.id}
              label={input.label}
            />
          );
        })}
      </div>
      <div>
        <div
          className={clsx(
            "title p-3 font-bold text-base border-b border-gray-200 text-center overflow-hidden text-ellipsis whitespace-nowrap"
          )}
        >
          User
        </div>
      </div>
    </div>
  );
};

export { User };
