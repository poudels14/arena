import {
  Show,
  createComputed,
  createMemo,
  createSelector,
  createSignal,
  useContext,
} from "solid-js";
import {
  useSharedWorkspaceContext,
  SharedWorkspaceContext,
} from "@portal/workspace-sdk";
import { Chatbox } from "./ChatBox";
import { ChatContext } from "./ChatContext";
import { ChatThread } from "./ChatThread";
import { Artifacts } from "./Artifacts";

type WorkspaceChatContext = NonNullable<
  ReturnType<SharedWorkspaceContext["getChatContext"]>
>;

const ChatIsland = (props: { onNewThread: () => void; hide?: boolean }) => {
  const { getChatContext, isChatIslandVisible } = useSharedWorkspaceContext();
  const { state, sendNewMessage } = useContext(ChatContext)!;
  const [chatBoxExpanded, setExpandChatBox] = createSignal(false);
  const [getActiveTab, setActiveTab] = createSignal("chat");
  const isTabActive = createSelector(getActiveTab);
  createComputed(() => {
    if (props.hide) {
      setExpandChatBox(false);
    }
  });
  return (
    <Show when={isChatIslandVisible() && getChatContext().length > 0}>
      <Show when={chatBoxExpanded()}>
        <div
          class="chat-island fixed bottom-0 left-0 right-0 w-full backdrop-blur-[1px]"
          classList={{
            "top-0": chatBoxExpanded(),
          }}
        >
          <div
            class="overlay w-full h-full bg-slate-400/40"
            onClick={(e) => {
              setExpandChatBox(false);
            }}
          ></div>
          <div class="absolute bottom-0 w-full flex justify-center pointer-events-none">
            <div class="island flex flex-col flex-1 min-w-[200px] max-w-[700px] bg-white border-t rounded-lg drop-shadow-md shadow-lg pointer-events-auto">
              <div class="flex text-xs shadow-sm text-gray-500">
                <div
                  class="px-4 py-1 border-transparent border-t border-r rounded-t cursor-pointer hover:bg-gray-100"
                  classList={{
                    "bg-gray-100  border-gray-200 text-gray-700":
                      isTabActive("chat"),
                  }}
                  onClick={() => setActiveTab("chat")}
                >
                  Chat
                </div>
                <div
                  class="px-4 py-1 border-transparent border-t border-r rounded-t cursor-pointer hover:bg-gray-100"
                  classList={{
                    "bg-gray-100 border-gray-200 text-gray-700":
                      isTabActive("artifacts"),
                  }}
                  onClick={() => setActiveTab("artifacts")}
                >
                  Artifacts
                </div>
              </div>
              <Show when={chatBoxExpanded()}>
                <div class="flex justify-center">
                  <div class="max-h-[500px] min-h-[225px] w-full">
                    <Show when={getActiveTab() == "chat"}>
                      <ChatThread
                        showDocument={() => {}}
                        removeBottomPadding={true}
                      />
                    </Show>
                    <Show when={getActiveTab() == "artifacts"}>
                      <div class="max-h-[225px] overflow-x-hidden overflow-y-auto scroll:w-1 thumb:rounded thumb:bg-gray-400">
                        <Artifacts />
                      </div>
                    </Show>
                  </div>
                </div>
              </Show>
              <div class="flex justify-center pb-6 space-y-1 pt-1 bg-gray-100 rounded-b-lg">
                <div class="flex-1 max-w-[650px]">
                  <Chatbox
                    threadId={state.activeThreadId()}
                    blockedBy={null}
                    sendNewMessage={(input) => {
                      sendNewMessage.mutate(input);
                    }}
                    onNewThread={props.onNewThread}
                    disableContextEdit={true}
                    onFocus={() => setExpandChatBox(true)}
                    autoFocus={true}
                    showContextBreadcrumb={true}
                    context={getChatContext()}
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </Show>
      <Show when={!chatBoxExpanded()}>
        <div class="absolute bottom-4 left-0 right-0 w-full backdrop-blur-[1px]">
          <div class="w-full flex justify-center">
            <Minimized
              context={getChatContext()}
              openChatBox={() => setExpandChatBox(true)}
            />
          </div>
        </div>
      </Show>
    </Show>
  );
};

const Minimized = (props: {
  context: WorkspaceChatContext;
  openChatBox: () => void;
}) => {
  const contextTitle = createMemo(() => {
    if (props.context.length == 0) {
      return "";
    }
    const { app, breadcrumbs } = props.context[0];
    return breadcrumbs.length > 0
      ? breadcrumbs[breadcrumbs.length - 1].title
      : app.name;
  });

  return (
    <div
      class="px-4 py-1.5 w-[300px] rounded-2xl bg-slate-300 text-gray-800 cursor-pointer select-none"
      onClick={props.openChatBox}
    >
      <div class="flex text-sm justify-center group">
        <div class="text-nowrap">Ask AI about</div>
        <div class="flex px-2 font-semibold space-x-1 overflow-hidden">
          <div class="overflow-hidden text-ellipsis text-nowrap">
            {contextTitle()}
          </div>
        </div>
      </div>
    </div>
  );
};

export { ChatIsland };
