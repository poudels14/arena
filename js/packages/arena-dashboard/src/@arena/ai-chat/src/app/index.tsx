import { useContext, createComputed } from "solid-js";
import { Routes, Route, useParams } from "@solidjs/router";
import { Sidebar } from "./sidebar";
import { ChatContext, ChatContextProvider } from "./chat/ChatContext";
import Chat from "./chat";

const App = () => {
  return (
    <Routes>
      <Route
        path="/chat/*"
        element={
          <ChatContextProvider>
            <div class="w-full h-screen flex flex-row">
              <Sidebar />
              <Chat />
            </div>

            <Routes>
              <Route path="/:channelId">
                <Route
                  path="/t/:threadId"
                  component={() => {
                    const params = useParams();
                    const { setChatChannel } = useContext(ChatContext)!;
                    createComputed(() => {
                      setChatChannel(params.channelId, params.threadId);
                    });
                    return null;
                  }}
                />
                <Route
                  path="/"
                  component={() => {
                    const params = useParams();
                    const { setChatChannel } = useContext(ChatContext)!;
                    createComputed(() => {
                      setChatChannel(params.channelId);
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
  );
};

export default App;
