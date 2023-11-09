import { Switch, Match, createComputed, lazy } from "solid-js";
import { Routes, Route, useParams } from "@solidjs/router";
// import { Workflow } from "@arena/sdk/workflow";
import { Sidebar } from "./sidebar";
import { ChatContextProvider } from "./chat/ChatContext";
import {
  AssistantContextProvider,
  useAssistantContext,
} from "./AssistantContext";

const Chat = lazy(() => import("./chat/index.tsx"));
const Configure = lazy(() => import("./configure/index.tsx"));

const Assistant = () => {
  return (
    <AssistantContextProvider>
      <Routes>
        {/* <Route path="/workflow" element={<Workflow />} /> */}
        <Route
          path="/*"
          element={
            <ChatContextProvider>
              <div class="w-full h-screen flex flex-row">
                <Sidebar />
                <AssistantTabs />
              </div>

              <Routes>
                <Route path="/:assistantId">
                  <Route
                    path="/configure"
                    component={() => {
                      const params = useParams();
                      const { setActiveAssistant } = useAssistantContext();
                      createComputed(() => {
                        setActiveAssistant({
                          assistantId: params.assistantId,
                          tab: "configure",
                        });
                      });
                      return null;
                    }}
                  />
                  <Route
                    path="/t/:threadId"
                    component={() => {
                      const params = useParams();
                      const { setActiveAssistant } = useAssistantContext();
                      createComputed(() => {
                        setActiveAssistant({
                          assistantId: params.assistantId,
                          tab: "chat",
                          threadId: params.threadId,
                        });
                      });
                      return null;
                    }}
                  />
                  <Route
                    path="/"
                    component={() => {
                      const params = useParams();
                      const { setActiveAssistant } = useAssistantContext();
                      createComputed(() => {
                        setActiveAssistant({
                          assistantId: params.assistantId,
                          tab: "chat",
                        });
                      });
                      return null;
                    }}
                  />
                </Route>
              </Routes>
            </ChatContextProvider>
          }
        />
      </Routes>
    </AssistantContextProvider>
  );
};

const AssistantTabs = () => {
  const { state } = useAssistantContext();
  return (
    <Switch>
      <Match when={state.activeTab() == "chat"}>
        <Chat />
      </Match>
      <Match when={state.activeTab() == "configure"}>
        <Configure />
      </Match>
    </Switch>
  );
};

export default Assistant;
