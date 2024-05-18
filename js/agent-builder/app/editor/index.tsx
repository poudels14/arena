import { useCallback, useMemo, useState } from "react";
import ReactFlow, {
  Background,
  Controls,
  Edge,
  Node,
  ReactFlowProvider,
  addEdge,
  useEdgesState,
  useNodesState,
  useOnSelectionChange,
  useReactFlow,
} from "reactflow";
import "reactflow/dist/style.css";
import { uniqueId } from "@portal/cortex/utils/uniqueId";

import "./style.css";
import { User } from "./nodes/User";
import { AgentInput } from "./nodes/AgentInput";
import { AgentNode } from "./nodes/AgentNode";
import { Toolbar } from "./toolbar";
import { EditorContextProvider } from "./Context";

type Graph = {
  nodes: (Node & { type: string })[];
  edges: (Edge & {
    sourceHandle: string;
    targetHandle: string;
  })[];
};

type AgentEditorProps = {
  agentNodes: any[];
  graph: Graph;
  // this should return true if changes were saved successfully
  saveGraph: (graph: Graph) => Promise<boolean>;
};

const AgentEditorCore = (props: AgentEditorProps) => {
  const nodeTypes = useMemo(
    () => ({
      agentNode: AgentNode,
      agentInput: AgentInput,
      user: User,
    }),
    []
  );

  const reactFlow = useReactFlow();
  const [nodes, setNodes, onNodesChange] = useNodesState(props.graph.nodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(props.graph.edges);
  const [selectedNodes, setSelectedNodes] = useState<string[]>([]);
  const [hasUnsavedChanged, setHasUnsavedChanged] = useState(false);
  useOnSelectionChange({
    onChange: ({ nodes, edges }) => {
      const selectedNodes = nodes.map((node) => node.id);
      setSelectedNodes(selectedNodes);
    },
  });

  const onConnect = useCallback((params: any) => {
    setEdges((prev) => {
      const newEdge = {
        id:
          params.source +
          "-" +
          params.sourceHandle +
          "-" +
          params.target +
          "-" +
          params.targetHandle,
        ...params,
      };
      if (newEdge.target == "user") {
        newEdge.type = "step";
      }
      const edges = addEdge(newEdge, prev);
      return edges;
    });
  }, []);

  const onDragOver = useCallback((event: any) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onDrop = useCallback(
    (event: any) => {
      event.preventDefault();
      const agentNodeId = event.dataTransfer.getData(
        "application/reactflow/agentNodeId"
      );
      if (typeof agentNodeId === "undefined" || !agentNodeId) {
        return;
      }

      const agentNode = props.agentNodes.find(
        (agentNode) => agentNode.id == agentNodeId
      );
      const position = reactFlow.screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });
      const newNode = {
        id: uniqueId(),
        type: "agentNode",
        position,
        data: {
          type: agentNode.id,
          label: agentNode.name,
          config: Object.fromEntries(
            agentNode.config.map((field: any) => {
              return [field.id, field.schema.default];
            })
          ),
        },
      };
      setNodes((prev) => prev.concat(newNode));
    },
    [reactFlow]
  );

  const isValidConnection = (connection: any) => {
    const sourceNode = nodes.find((n) => n.id == connection.source);
    const sourceAgentNode = props.agentNodes.find(
      (n) => n.id == sourceNode?.data.type
    );

    const targetNode = nodes.find((n) => n.id == connection.target);
    const targetAgentNode = props.agentNodes.find(
      (n) => n.id == targetNode?.data.type
    );

    const sourceHandleOutput = sourceAgentNode.outputs.find(
      (output: any) => output.id == connection.sourceHandle
    );
    const targetHandleInput = targetAgentNode.inputs.find(
      (input: any) => input.id == connection.targetHandle
    );

    return sourceHandleOutput.schema.type == targetHandleInput.schema.type;
  };

  return (
    <EditorContextProvider
      value={{
        agentNodes: props.agentNodes,
        setNodes,
        selectedNodes,
        saveChanges: () => {
          props
            .saveGraph({
              nodes: nodes as Graph["nodes"],
              edges: edges as Graph["edges"],
            })
            .then((saved) => {
              setHasUnsavedChanged(!saved);
            });
        },
        hasUnsavedChanged,
      }}
    >
      <div className="w-screen h-screen flex bg-slate-50 text-xs">
        <div className="flex-1">
          <ReactFlow
            className="agent-editor"
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            nodeTypes={nodeTypes}
            defaultViewport={{
              x: 100,
              y: 200,
              zoom: 0.9,
            }}
            proOptions={{ hideAttribution: true }}
            onConnect={onConnect}
            isValidConnection={isValidConnection}
            onDragOver={onDragOver}
            onDrop={onDrop}
          >
            <Controls />
            <Background />
          </ReactFlow>
        </div>
        <Toolbar />
      </div>
    </EditorContextProvider>
  );
};

const AgentEditor = (props: AgentEditorProps) => {
  return (
    <ReactFlowProvider>
      <AgentEditorCore {...props} />
    </ReactFlowProvider>
  );
};
export { AgentEditor };
