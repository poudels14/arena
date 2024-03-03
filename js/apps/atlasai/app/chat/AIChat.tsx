import { Show, createMemo, createSignal, useContext } from "solid-js";
import { useNavigate } from "@portal/solid-router";
import { DocumentViewer } from "./DocumentViewer";
import { ChatContext } from "./ChatContext";
import { Chatbox } from "./ChatBox";
import { ChatThread } from "./ChatThread";
import { useSharedWorkspaceContext } from "@portal/workspace-sdk";

const AIChat = () => {
  const navigate = useNavigate();
  const { activeWorkspace, getChatContext } = useSharedWorkspaceContext();
  const { state, sendNewMessage, activeChatThread } = useContext(ChatContext)!;
  const [drawerDocument, setDrawerDocument] = createSignal<any>(null);
  const contextSelection = createMemo(() => {
    const apps = activeWorkspace.apps();
    if (!apps) {
      return [];
    }
    return apps
      .filter((app) => {
        return app.template.id == "portal-drive";
      })
      .map((app) => {
        return {
          app: {
            id: app.id,
            name: "Drive",
          },
          breadcrumbs: [],
          selection: undefined,
        };
      });
  }, []);
  return (
    <div class="chat relative flex-1 h-full min-w-[300px]">
      <div class="flex h-full">
        <div class="flex-1">
          <ChatThread
            showDocument={() => {}}
            contextSelection={contextSelection()}
          />
        </div>
      </div>
      <div class="chatbox-container absolute bottom-0 w-full flex justify-center pointer-events-none">
        <div class="flex-1 -ml-6 min-w-[200px] max-w-[750px] rounded-lg pointer-events-auto backdrop-blur-xl bg-gray-400/10 space-y-1">
          <div class="pb-4 bg-gradient-to-b from-transparent to-white rounded">
            <Chatbox
              threadId={state.activeThreadId()!}
              blockedBy={activeChatThread.blockedBy()}
              sendNewMessage={(input) => sendNewMessage.mutate(input)}
              onNewThread={() => navigate(`/`)}
              autoFocus={true}
              showContextBreadcrumb={false}
              context={getChatContext()}
            />
          </div>
        </div>
      </div>
      <Show when={drawerDocument()}>
        <DocumentViewer
          document={drawerDocument()}
          onClose={() => setDrawerDocument(null)}
        />
      </Show>
    </div>
  );
};

export { AIChat };
