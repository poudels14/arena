import { useMemo } from "react";
import { useNodes } from "reactflow";
import clsx from "clsx";

import { NodeEditor } from "./NodeEditor";
import NodeList from "./NodeList";
import { useEditorContext } from "../Context";

const Toolbar = () => {
  const editor = useEditorContext();
  const nodes = useNodes<any>();
  const selectedNode = useMemo(
    () => nodes.find((n) => editor.selectedNodes.includes(n.id)),
    [editor.selectedNodes]
  )!;

  return (
    <div
      className={clsx(
        "h-full top-4 right-1 w-72 text-gray-700 bg-slate-100 border-l border-gray-200/20 shadow",
        !selectedNode && "h-8",
        selectedNode && "h-[calc(100vh-theme(spacing.8))]"
      )}
    >
      <TopBar />
      <div>
        {selectedNode && <NodeEditor node={selectedNode} />}
        {!selectedNode && <NodeList />}
      </div>
    </div>
  );
};

const TopBar = () => {
  const editor = useEditorContext();
  return (
    <div className="flex justify-left py-2 px-2 bg-gray-50 border-b border-gray-300 space-x-2">
      <button
        type="button"
        className="px-3 py-1 rounded bg-indigo-500 hover:bg-indigo-600 text-gray-50"
        onClick={() => editor.saveChanges()}
      >
        Save
      </button>
    </div>
  );
};

export { Toolbar };
