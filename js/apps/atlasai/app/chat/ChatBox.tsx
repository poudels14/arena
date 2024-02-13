import {
  For,
  Match,
  Show,
  Switch,
  createComputed,
  createEffect,
  createMemo,
  createSignal,
} from "solid-js";
import {
  HiOutlinePaperAirplane,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineChevronRight,
} from "solid-icons/hi";
import { Chat } from "../types";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { SharedWorkspaceContext } from "@portal/workspace-sdk";

type ChatQueryContext = NonNullable<
  ReturnType<SharedWorkspaceContext["getChatContext"]>
>;

const Chatbox = (props: {
  threadId: string | undefined;
  blockedBy: Chat.Thread["blockedBy"];
  sendNewMessage: (msg: {
    id: string;
    threadId: string;
    message: { content: string };
    context?: ChatQueryContext | null;
    isNewThread: boolean;
  }) => void;
  onFocus?: () => void;
  autoFocus?: boolean;
  disableContextEdit?: boolean;
  context?: ChatQueryContext | null;
}) => {
  const [getMessage, setMessage] = createSignal("");
  const [getTextareaHeight, setTextareaHeight] = createSignal(
    "22px" /* height of 1 line + border */
  );

  // TODO(sagar): move this to @arena/components Textarea
  let textareaRef: any;
  let textareaTextRef: any;

  /**
   * Focus on text box when thread changes
   */
  createEffect(() => {
    void props.threadId;
    if (props.autoFocus) {
      textareaRef?.focus();
    }
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
    props.sendNewMessage({
      id: uniqueId(19),
      threadId: props.threadId || uniqueId(19),
      message: {
        content: getMessage(),
      },
      context: props.context,
      isNewThread: !Boolean(props.threadId),
    });
    setMessage("");
    textareaRef?.focus();
  };

  const keydownHandler = (e: any) => {
    const value = e.target.value;
    if (
      e.key == "Enter" &&
      !e.shiftKey &&
      !props.blockedBy &&
      value.trim().length > 0
    ) {
      submitForm();
      e.preventDefault();
      e.stopPropagation();
    }
  };

  return (
    <div class="chat-box min-w-[200px] max-w-[650px] space-y-1">
      <Show when={props.context}>
        <SelectChatContext
          context={props.context!}
          disableContextEdit={props.disableContextEdit}
        />
      </Show>
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
            onFocus={() => {
              props.onFocus?.();
            }}
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
              <HiOutlinePaperAirplane size="14px" />
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

const SelectChatContext = (props: {
  context: ChatQueryContext;
  disableContextEdit?: boolean;
}) => {
  const visibleBreadCrumbs = createMemo(() => {
    return props.context.breadcrumbs.slice(
      props.context.breadcrumbs.length - 2
    );
  });
  return (
    <div class="flex px-2 text-sm font-semibold space-x-1 text-brand-12/90">
      <div class="flex overflow-hidden">
        <Show when={!props.disableContextEdit}>
          <div class="flex p-1 rounded-l bg-gray-200 hover:bg-gray-300 text-gray-700 cursor-pointer">
            <HiOutlineXMark size={16} />
          </div>
        </Show>
        <div class="flex rounded bg-gray-50 overflow-hidden">
          <div
            class="flex px-2 cursor-pointer space-x-1 rounded"
            classList={{
              "hover:bg-gray-200": !props.disableContextEdit,
            }}
          >
            <div class="py-1">
              <Switch>
                <Match when={props.context.app.icon == "folder"}>
                  <HiOutlineFolderOpen size={16} />
                </Match>
              </Switch>
            </div>
            <div class="py-0.5 text-nowrap">{props.context.app.name}</div>
          </div>
          <Show when={props.context.breadcrumbs.length > 0}>
            <div class="py-1">
              <HiOutlineChevronRight size={16} />
            </div>
          </Show>
          <Show
            when={
              visibleBreadCrumbs().length < props.context.breadcrumbs.length
            }
          >
            <div>...</div>
            <div class="py-1">
              <HiOutlineChevronRight size={16} />
            </div>
          </Show>
          <For each={visibleBreadCrumbs()}>
            {(breadcrumb, index) => {
              return (
                <>
                  <div
                    class="flex px-2 cursor-pointer space-x-1 rounded overflow-hidden text-nowrap"
                    classList={{
                      "hover:bg-gray-200": !props.disableContextEdit,
                    }}
                  >
                    <div class="py-0.5 overflow-hidden text-ellipsis">
                      {breadcrumb.title}
                    </div>
                  </div>
                  <Show when={index() < visibleBreadCrumbs().length - 1}>
                    <div class="py-1">
                      <HiOutlineChevronRight size={16} />
                    </div>
                  </Show>
                </>
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
};

export { Chatbox };
