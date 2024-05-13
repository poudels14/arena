export type Node = {
  id: string;
  type: string;
  config: any;
};

export type Edge = {
  id: string;
  from: {
    node: string;
    outputKey: string;
  };
  to: {
    node: string;
    inputKey: string;
  };
};

export type Graph = {
  nodes: Node[];
  edges: Edge[];
};
