import { useContext, Show, createMemo, For } from "solid-js";
import { A } from "@solidjs/router";
import { ChatContext } from "./ChatContext";

const AIThreads = () => {
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
      <div class="w-52 h-full flex flex-col border-r text-sm">
        <div class="px-1">
          <div class="py-1 h-7 rounded-sm font-medium text-center bg-brand-12/90 text-accent-1">
            Threads
          </div>
        </div>
        <div class="flex-1 py-4 overflow-y-auto no-scrollbar">
          <For each={threads()}>
            {(thread, index) => {
              return (
                <A href={`/chat/default/t/${thread.id}`} class="group">
                  <div class="py-1 px-2 hover:bg-brand-10/10 group-[.active]:bg-brand-10/20">
                    {index()}. {thread.title}
                  </div>
                </A>
              );
            }}
          </For>
        </div>
      </div>
    </Show>
  );
};

export { AIThreads };
