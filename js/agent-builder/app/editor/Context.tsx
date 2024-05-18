import React, { createContext, useContext } from "react";
import { Node } from "reactflow";

import { AgentNode } from "./AgentNodes";

type EditorContext = {
  setNodes: React.Dispatch<
    React.SetStateAction<Node<any, string | undefined>[]>
  >;
  agentNodes: AgentNode[];
  selectedNodes: string[];
  saveChanges: () => void;
  hasUnsavedChanged: boolean;
};

const EditorContext = createContext<EditorContext>({} as EditorContext);
const useEditorContext = () => useContext(EditorContext)!;

const EditorContextProvider = EditorContext.Provider;
export { EditorContextProvider, useEditorContext };
