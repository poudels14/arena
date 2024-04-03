import {
  Match,
  Show,
  Switch,
  createMemo,
  createSignal,
  useContext,
} from "solid-js";
import { useNavigate } from "@portal/solid-router";
import { DocumentViewer } from "./DocumentViewer";
import { ChatContext } from "./ChatContext";
import { Chatbox } from "./ChatBox";
import { ChatThread } from "./ChatThread";
import { useSharedWorkspaceContext } from "@portal/workspace-sdk";

const AIChat = () => {
  const navigate = useNavigate();
  const { activeWorkspace, getChatContext } = useSharedWorkspaceContext();
  const { state, sendNewMessage, getActiveChatThread } =
    useContext(ChatContext)!;
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
      <Switch>
        <Match when={activeWorkspace.models()?.length > 0}>
          <div class="flex h-full">
            <div class="flex-1">
              <ChatThread
                showDocument={() => {}}
                contextSelection={contextSelection()}
              />
            </div>
          </div>
          <div class="chatbox-container absolute bottom-0 w-full flex justify-center pointer-events-none">
            <div class="flex-1 -ml-6 min-w-[200px] max-w-[750px] rounded-lg pointer-events-auto backdrop-blur-xl space-y-1">
              <div class="mb-4 bg-gray-400/10 rounded">
                <Chatbox
                  threadId={state.activeThreadId()!}
                  blockedBy={getActiveChatThread().blockedBy()}
                  sendNewMessage={(input) => sendNewMessage.mutate(input)}
                  onNewThread={() => navigate(`/`)}
                  autoFocus={true}
                  showContextBreadcrumb={false}
                  context={getChatContext()}
                />
              </div>
            </div>
          </div>
        </Match>
        <Match when={true}>
          <div class="h-full flex flex-col justify-center space-y-1">
            <div class="text-center text-xl font-semibold text-gray-800">
              No AI model found.
            </div>
            <div class="text-center text-lg font-medium text-gray-600">
              Add a model in Settings.
            </div>
          </div>
        </Match>
      </Switch>

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
