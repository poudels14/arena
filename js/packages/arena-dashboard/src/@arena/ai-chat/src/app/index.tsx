import { Show, createResource } from "solid-js";
import { useAppContext } from "@arena/sdk/app";
import { Sidebar } from "./Sidebar";
import { Document } from "./types";
import { Chat } from "./Chat";
import { ChatContextProvider } from "./ChatContext";

const App = (props: any) => {
  const { router } = useAppContext();
  const [getDocuments] = createResource(
    async () => (await router.get<Document[]>("/documents")).data
  );

  return (
    <Show when={getDocuments()}>
      <div class="w-full h-screen flex flex-row">
        <Sidebar documents={getDocuments()!} />
        <div class="flex-1">
          <ChatContextProvider>
            <Chat />
          </ChatContextProvider>
        </div>
      </div>
    </Show>
  );
};

export default App;
