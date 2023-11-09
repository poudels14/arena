import { Store, StoreSetter, createStore } from "@arena/solid-store";
import { createContext, useContext, batch } from "solid-js";

export type AssistantState = {
  activeAssistantId: string | null;
  activeTab: string | null;
  activeThreadId: string | null;
};

type AssistantContext = {
  state: Store<AssistantState>;
  setState: StoreSetter<AssistantState>;
  setActiveAssistant: (options: {
    assistantId: string;
    tab: string;
    threadId?: string;
  }) => void;
};

const AssistantContext = createContext<AssistantContext>();

const useAssistantContext = () => useContext(AssistantContext)!;

const AssistantContextProvider = (props: { children: any }) => {
  const [state, setState] = createStore<AssistantState>({
    activeAssistantId: "default",
    activeTab: null,
    activeThreadId: null,
  });

  const setActiveAssistant = (options: {
    assistantId: string;
    tab: string;
    threadId?: string;
  }) => {
    batch(() => {
      setState("activeAssistantId", options.assistantId || "default");
      setState("activeTab", options.tab || null);
      setState("activeThreadId", options.threadId || null);
    });
  };

  return (
    <AssistantContext.Provider value={{ state, setState, setActiveAssistant }}>
      {props.children}
    </AssistantContext.Provider>
  );
};

export { AssistantContextProvider, useAssistantContext };
