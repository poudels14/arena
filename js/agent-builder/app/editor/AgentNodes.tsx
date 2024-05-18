import { createContext, useContext } from "react";

type AgentNode = {
  id: string;
  name: string;
  icon?: string;
  config: { id: string; label: string }[];
  inputs: { id: string; label: string }[];
  outputs: { id: string; label: string }[];
};

type AgentNodeContext = {
  nodes: AgentNode[];
};

const AgentNodeContext = createContext<AgentNodeContext>(
  {} as AgentNodeContext
);
const useAgentNodeContext = () => useContext(AgentNodeContext)!;

const AgentNodeContextProvider = (props: {
  value: Pick<AgentNodeContext, "nodes">;
  children: any;
}) => {
  return (
    <AgentNodeContext.Provider value={props.value}>
      {props.children}
    </AgentNodeContext.Provider>
  );
};

export { AgentNodeContextProvider, useAgentNodeContext };
export type { AgentNode };
