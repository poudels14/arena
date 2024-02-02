import { Show, createMemo, createSignal, useContext } from "solid-js";
import {
  useSharedWorkspaceContext,
  SharedWorkspaceContext,
} from "@portal/workspace-sdk";
import { Chatbox } from "./ChatBox";
import { ChatContext } from "./ChatContext";
import { ChatThread } from "./ChatThread";

type WorkspaceChatContext = NonNullable<
  ReturnType<SharedWorkspaceContext["getChatContext"]>
>;

const ChatIsland = () => {
  const { getChatContext } = useSharedWorkspaceContext();
  const { sendNewMessage } = useContext(ChatContext)!;
  const [chatBoxExpanded, setExpandChatBox] = createSignal(false);
  return (
    <Show when={getChatContext()}>
      <Show when={chatBoxExpanded()}>
        <div
          class="absolute bottom-0 left-0 right-0 w-full backdrop-blur-[1px]"
          classList={{
            "top-0": chatBoxExpanded(),
          }}
        >
          <div
            class="overlay w-full h-full"
            onClick={(e) => {
              setExpandChatBox(false);
            }}
          ></div>
          <div class="absolute bottom-2 w-full flex justify-center pointer-events-none">
            <div class="flex flex-col flex-1 min-w-[200px] max-w-[650px] bg-white border-t rounded-lg drop-shadow-md shadow-lg pointer-events-auto">
              <Show when={chatBoxExpanded()}>
                <div class="flex justify-center">
                  <div class="min-h-[450px] w-full">
                    <ChatThread showDocument={() => {}} />
                  </div>
                </div>
              </Show>
              <div class="space-y-1 pt-1 bg-gray-100 rounded-b-lg">
                <Chatbox
                  threadId={undefined}
                  blockedBy={null}
                  sendNewMessage={(input) => {
                    sendNewMessage.mutate(input);
                  }}
                  hideClearContextButton={true}
                  onFocus={() => setExpandChatBox(true)}
                  autoFocus={true}
                  context={getChatContext()}
                />
              </div>
            </div>
          </div>
        </div>
      </Show>
      <Show when={!chatBoxExpanded()}>
        <div class="absolute bottom-2 left-0 right-0 w-full backdrop-blur-[1px]">
          <div class="w-full flex justify-center">
            <Minimized
              context={getChatContext()!}
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
    const { app, breadcrumbs } = props.context;
    return breadcrumbs.length > 0
      ? breadcrumbs[breadcrumbs.length - 1].title
      : app.name;
  });

  return (
    <div
      class="px-4 py-1.5 w-[300px] rounded-2xl bg-brand-12/80 text-white cursor-pointer select-none"
      onClick={props.openChatBox}
    >
      <div class="flex text-sm justify-center group">
        <div class="text-nowrap">Search or ask about</div>
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
