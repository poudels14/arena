import { useContext, Show, createMemo, For } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { ChatContext } from "./ChatContext";

const AIThreads = () => {
  const navigate = useNavigate();
  const { state } = useContext(ChatContext)!;
  const threads = createMemo(() => {
    const channelId = state.activeChannelId();
    const threads = Object.values(state.threads());
    const filtered = threads
      .filter((t) => t.channelId == channelId)
      .sort((a, b) => b.timestamp - a.timestamp);

    return filtered;
  });

  return (
    <Show when={state.activeChannelId()}>
      <div class="w-60 h-full flex flex-col border-r border-gray-100 text-sm">
        <div class="px-1">
          <div class="py-1 h-7 rounded-sm font-medium text-center bg-brand-12/90 text-accent-1">
            Threads
          </div>
        </div>
        <div class="flex-1 py-2 text-[0.75rem] text-gray-700 overflow-y-auto no-scrollbar divide-y divide-gray-200">
          <For each={threads()}>
            {(thread) => {
              return (
                <div
                  class="py-1.5 px-2 flex cursor-pointer hover:bg-brand-10/10"
                  classList={{
                    "bg-brand-10/20": state.activeThreadId() == thread.id,
                  }}
                  onClick={() => {
                    navigate(`/chat/default/t/${thread.id}`);
                  }}
                >
                  <span class="whitespace-nowrap text-ellipsis overflow-hidden">
                    {thread.title}
                  </span>
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </Show>
  );
};

export { AIThreads };
