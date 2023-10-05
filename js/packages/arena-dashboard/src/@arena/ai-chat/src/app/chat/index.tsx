import { useContext } from "solid-js";
import { AIChat } from "./AIChat";
import { AIThreads } from "./AIThreads";
import { ChatContext } from "./ChatContext";

const Chat = () => {
  const { state } = useContext(ChatContext)!;
  return (
    <div class="flex-1 flex">
      <AIThreads />
      <AIChat
        channelId={state.activeChannelId()!}
        threadId={state.activeThreadId()!}
      />
    </div>
  );
};

export default Chat;
