import { useMemo } from "react";
import { HiLink } from "react-icons/hi";

import { useEditorContext } from "../Context";

const NodeList = () => {
  const editor = useEditorContext();
  const agentNodes = useMemo(() => {
    return editor.agentNodes.filter(
      (n) => n.id != "@core/input" && n.id != "@core/user"
    );
  }, [editor.agentNodes]);

  const onDragStart = (event: any, node: any) => {
    event.dataTransfer.setData("application/reactflow/agentNodeId", node.id);
    event.dataTransfer.effectAllowed = "move";
  };

  return (
    <div className="w-full h-full">
      <div className="w-full py-1 font-semibold text-base text-center border-b border-gray-300">
        Nodes
      </div>
      <div className="text-sm space-y-1">
        {agentNodes.map((node, index) => {
          return (
            <div
              key={index}
              className="flex px-2 py-2 items-center even:bg-gray-200 cursor-grab space-x-1"
              draggable
              onDragStart={(event) => onDragStart(event, node)}
            >
              <div className="text-gray-500">
                {node.icon && (
                  <div
                    className="flex w-3 h-3 justify-center items-center"
                    dangerouslySetInnerHTML={{ __html: node.icon }}
                  ></div>
                )}
                {!node.icon && (
                  <div className="flex w-3 h-3 justify-center items-center">
                    <HiLink />
                  </div>
                )}
              </div>
              <div className="text-gray-800">{node.name}</div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default NodeList;
