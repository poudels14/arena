import {
  Index,
  Show,
  createComputed,
  createEffect,
  createMemo,
  createSignal,
  onMount,
  useContext,
} from "solid-js";
import { InlineIcon } from "@arena/components";
import { Markdown } from "@arena/components/markdown";
import { Marked } from "marked";
import SendIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/send-message";
import AddIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/plus";
import { ChatContext } from "./ChatContext";

const Chat = () => {
  const { state } = useContext(ChatContext)!;
  let chatMessagesContainerRef: any;
  let chatMessagesRef: any;

  return (
    <div class="chat relative h-full">
      <div
        ref={chatMessagesContainerRef}
        class="flex justify-center h-full overflow-y-auto"
      >
        <div class="flex-1 max-w-[650px]">
          <div
            ref={chatMessagesRef}
            class="chat-messages pb-24 py-2 space-y-5 text-sm text-accent-12/80"
          >
            <Index each={state.activeSession.messages()}>
              {(_, index) => {
                // Note(sagar): use state directly to only update message
                // content element when streaming
                const m = state.activeSession.messages[index];
                if (index == state.activeSession.messages().length - 1) {
                  createEffect(() => {
                    void m.message();
                    // Note(sagar): scroll to the bottom. Need to do it after
                    // the last message is rendered
                    const containerHeight = parseFloat(
                      getComputedStyle(chatMessagesRef).height
                    );
                    chatMessagesContainerRef.scrollTo(
                      0,
                      containerHeight + 100_000
                    );
                  });
                }

                const marked = new Marked({});
                const tokens = createMemo(() => marked.lexer(m.message()));

                return (
                  <div
                    class="px-3 py-1 rounded-sm"
                    classList={{
                      "bg-blue-100": m.role() == "user",
                      "bg-brand-3": m.role() == "ai",
                      "border border-red-700": m.streaming(),
                    }}
                    data-message-id={m.id()}
                  >
                    <div
                      class="message"
                      style={"letter-spacing: 0.1px; word-spacing: 1px"}
                    >
                      <Markdown tokens={tokens()} />
                    </div>
                  </div>
                );
              }}
            </Index>
          </div>
        </div>
      </div>
      <div class="chatbox-container absolute bottom-2 w-full flex justify-center pointer-events-none">
        <div class="flex-1 max-w-[560px] rounded-lg pointer-events-auto backdrop-blur-xl">
          <div class="flex p-2 flex-row text-accent-11">
            <Show when={state.activeSession.messages().length > 0}>
              <div class="new-chat flex pr-2 text-xs font-normal text-brand-12/80 border border-brand-12/50 rounded align-middle cursor-pointer select-none bg-white shadow-2xl">
                <InlineIcon size="20px" class="py-1">
                  <path d={AddIcon[0]} />
                </InlineIcon>
                <div class="leading-5">New chat</div>
              </div>
            </Show>
          </div>
          <Chatbox isGeneratingResponse={state.isGeneratingResponse} />
        </div>
      </div>
    </div>
  );
};

const Chatbox = (props: any) => {
  const { sendNewMessage } = useContext(ChatContext)!;
  const [getMessage, setMessage] = createSignal("");
  const [getTextareaHeight, setTextareaHeight] = createSignal(
    "22px" /* height of 1 line + border */
  );

  // TODO(sagar): move this to @arena/components Textarea
  let textareaRef: any;
  let textareaTextRef: any;
  onMount(() => {
    textareaRef.focus();
  });

  createComputed(() => {
    const msg = getMessage();
    if (!textareaTextRef) return;
    textareaTextRef.innerText = msg;
    var s = getComputedStyle(textareaTextRef) as any;
    let height =
      Math.max(
        20 /* height of 1 line */,
        parseFloat(s.height) -
          parseFloat(s.paddingTop) -
          parseFloat(s.paddingBottom)
      ) + 2; /* border */
    if (msg.substring(msg.length - 1) == "\n") {
      height += parseFloat(s.lineHeight);
    }
    setTextareaHeight(height + "px");
  });

  const submitForm = () => {
    sendNewMessage(getMessage());
    setMessage("");
    textareaRef?.focus();
  };

  const keydownHandler = (e: any) => {
    const value = e.target.value;
    if (
      e.key == "Enter" &&
      !e.shiftKey &&
      !props.isGeneratingResponse() &&
      value.trim().length > 0
    ) {
      submitForm();
      e.preventDefault();
      e.stopPropagation();
    }
  };

  return (
    <div class="relative py-2 rounded-lg bg-brand-12/90 shadow-lg backdrop-blur-sm">
      <form
        class="p-0 m-0"
        onSubmit={(e) => {
          submitForm();
          e.preventDefault();
        }}
      >
        <textarea
          ref={textareaRef}
          placeholder="Send a message"
          class="w-full max-h-[180px] px-4 text-sm text-white bg-transparent outline-none resize-none placeholder:text-gray-400"
          style={{
            height: getTextareaHeight(),
            "--uikit-scrollbar-w": "3px",
            "--uikit-scrollbar-track-bg": "transparent",
            "--uikit-scrollbar-track-thumb": "rgb(210, 210, 210)",
          }}
          value={getMessage()}
          onInput={(e: any) => setMessage(e.target.value)}
          onkeydown={keydownHandler}
        ></textarea>
        <div
          class="absolute top-0 px-4 text-sm opacity-0 select-none pointer-events-none whitespace-break-spaces"
          ref={textareaTextRef}
        />
        <div
          class="absolute bottom-2 right-0 px-2"
          classList={{
            "text-white": getMessage().trim().length > 0,
            "text-gray-500": true,
          }}
        >
          <button class="p-1 bg-brand-10/20 rounded outline-none">
            <InlineIcon size="14px">
              <path d={SendIcon[0]} />
            </InlineIcon>
          </button>
        </div>
      </form>
    </div>
  );
};

export { Chat };
