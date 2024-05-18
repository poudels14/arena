import { useEffect, useMemo, useRef } from "react";
import {
  Handle,
  Position,
  useReactFlow,
  useUpdateNodeInternals,
} from "reactflow";

type NodeWithhandleProps = {
  nodeId: string;
  nodeCoords: { top: number; left: number };
  type: "source" | "target";
  position: Position;
  id: string;
  label: string;
};

const NodeHandle = (props: NodeWithhandleProps) => {
  const reactflow = useReactFlow();
  const updateNodeInternals = useUpdateNodeInternals();
  const ref = useRef<HTMLDivElement>(null);

  const style = useMemo(() => {
    const bounds = ref.current?.getBoundingClientRect();
    if (!bounds) {
      return {};
    }

    const { zoom } = reactflow.getViewport();
    const offset = {
      x: (bounds.left - props.nodeCoords.left) / zoom + bounds.width / 2 / zoom,
      y: (bounds.top - props.nodeCoords.top) / zoom + bounds.height / 2 / zoom,
    };

    updateNodeInternals(props.nodeId);
    setTimeout(() => {
      updateNodeInternals(props.nodeId);
    }, 1000);
    if (props.position == Position.Left || props.position == Position.Right) {
      return {
        top: offset.y + "px",
      };
    } else {
      return {
        left: offset.x + "px",
      };
    }
  }, [ref.current, props.position]);

  useEffect(() => {
    setTimeout(() => {
      updateNodeInternals(props.nodeId);
    }, 1000);
  }, []);

  return (
    <div ref={ref} className="handle">
      <div className="pl-2">{props.label}</div>
      <Handle
        type={props.type}
        id={props.id}
        position={props.position}
        style={style}
      />
    </div>
  );
};

export { NodeHandle };
