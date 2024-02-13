import { Show, createSignal, useContext } from "solid-js";
import { HiOutlinePlus } from "solid-icons/hi";
import { useNavigate } from "@portal/solid-router";
import { DocumentViewer } from "./DocumentViewer";
import { ChatContext } from "./ChatContext";
import { Chatbox } from "./ChatBox";
import { ChatThread } from "./ChatThread";

const AIChat = () => {
  const navigate = useNavigate();
  const { state, sendNewMessage, activeChatThread } = useContext(ChatContext)!;
  const [drawerDocument, setDrawerDocument] = createSignal<any>(null);
  return (
    <div class="chat relative flex-1 h-full min-w-[300px]">
      <div class="flex h-full">
        <div class="flex-1">
          <ChatThread showDocument={() => {}} />
        </div>
      </div>
      <div class="chatbox-container absolute bottom-2 w-full flex justify-center pointer-events-none">
        <div class="flex-1 -mr-10 min-w-[200px] max-w-[650px] rounded-lg pointer-events-auto backdrop-blur-xl bg-gray-400/10 space-y-1">
          <Show when={Boolean(state.activeThreadId())}>
            <div class="flex px-2 pt-2 flex-row text-accent-11">
              <div class="new-chat flex pr-2 text-xs font-normal text-brand-12/80 border border-brand-12/50 rounded align-middle cursor-pointer select-none bg-white shadow-2xl">
                <HiOutlinePlus size="20px" class="py-1" />
                <div class="leading-5" onClick={() => navigate(`/`)}>
                  New thread
                </div>
              </div>
            </div>
          </Show>
          <Chatbox
            threadId={state.activeThreadId()!}
            blockedBy={activeChatThread.blockedBy()}
            sendNewMessage={(input) => sendNewMessage.mutate(input)}
          />
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
