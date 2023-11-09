import { AIChat } from "./AIChat";
import { AIThreads } from "./AIThreads";
import { useAssistantContext } from "../AssistantContext";

const Chat = () => {
  const { state } = useAssistantContext();
  return (
    <div class="flex-1 flex">
      <AIThreads />
      <AIChat
        assistantId={state.activeAssistantId()!}
        threadId={state.activeThreadId()!}
      />
    </div>
  );
};

export default Chat;
