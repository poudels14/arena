import {
  Match,
  Show,
  Switch,
  createComputed,
  createMemo,
  createSignal,
  lazy,
  useContext,
} from "solid-js";
import { Resizable } from "corvu/resizable";
import { useNavigate } from "@portal/solid-router";
import { useSharedWorkspaceContext } from "@portal/workspace-sdk";
import { DocumentViewer } from "./DocumentViewer";
import { ChatContext } from "./ChatContext";
import { Chatbox } from "./ChatBox";
import { ChatThread } from "./ChatThread";

const AgentPanel = lazy(() => import("./agentpanel"));

const AIChat = () => {
  const { activeWorkspace } = useSharedWorkspaceContext();
  const [drawerDocument, setDrawerDocument] = createSignal<any>(null);
  return (
    <div class="chat flex relative flex-1 h-full overflow-hidden">
      <Switch>
        <Match when={activeWorkspace.models()?.length > 0}>
          <Resizable class="flex-1">
            <ThreadPanel />
          </Resizable>
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

const ThreadPanel = () => {
  const navigate = useNavigate();
  const { activeWorkspace, getChatContext } = useSharedWorkspaceContext();
  const { state, sendNewMessage, getActiveChatThread } =
    useContext(ChatContext)!;

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

  const context = Resizable.useContext();
  const showAgentPanel = createMemo(() => {
    return getActiveChatThread().metadata.agent?.layout!()?.includes(
      "show-agentpanel"
    );
  });

  createComputed(() => {
    context.setSizes(showAgentPanel() ? [0.3, 0.7] : [1, 0]);
  });

  return (
    <>
      <Resizable.Panel minSize={0.15} initialSize={1}>
        <div class="relative flex-1 h-full min-w-[300px] overflow-hidden">
          <div class="text-xs">
            <ChatThread
              showDocument={() => {}}
              contextSelection={contextSelection()}
            />
          </div>
          <div class="chatbox-container absolute bottom-0 w-full px-4 flex justify-center pointer-events-none">
            <div class="flex-1 min-w-[200px] max-w-[750px] rounded-lg pointer-events-auto backdrop-blur-xl space-y-1">
              <div class="mb-4 bg-gray-400/10 rounded">
                <Chatbox
                  threadId={state.activeThreadId()!}
                  blockedBy={getActiveChatThread().blockedBy!()}
                  sendNewMessage={(input) => sendNewMessage.mutate(input)}
                  onNewThread={() => navigate(`/`)}
                  autoFocus={true}
                  showContextBreadcrumb={false}
                  context={getChatContext()}
                />
              </div>
            </div>
          </div>
        </div>
      </Resizable.Panel>
      <Resizable.Handle
        class="basis-1 bg-slate-100"
        classList={{
          hidden: context.sizes()[1] == 0,
        }}
      />
      <Resizable.Panel
        collapsible
        // TODO: this isn't working; fix corvu lib to make this adjustable
        // after the first render
        collapsedSize={showAgentPanel() ? 0.3 : 0}
        initialSize={0}
        minSize={0.15}
      >
        <div
          class="flex-1 h-full overflow-auto bg-slate-50"
          classList={{
            hidden: context.sizes()[1] == 0,
          }}
        >
          <AgentPanel />
        </div>
      </Resizable.Panel>
    </>
  );
};

export { AIChat };
