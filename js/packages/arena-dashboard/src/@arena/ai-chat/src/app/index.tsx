import { Sidebar } from "./sidebar";
import { Chat } from "./Chat";
import { ChatContextProvider } from "./ChatContext";

const App = (props: any) => {
  return (
    <ChatContextProvider>
      <div class="w-full h-screen flex flex-row">
        <Sidebar />
        <div class="flex-1">
          <Chat />
        </div>
      </div>
    </ChatContextProvider>
  );
};

export default App;
