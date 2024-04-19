import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
} from "solid-js";
import {
  HiOutlinePaperAirplane,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineChevronRight,
  HiOutlinePlus,
} from "solid-icons/hi";
import { Chat } from "../types";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { SharedWorkspaceContext } from "@portal/workspace-sdk";
import { adjustTextareaHeight } from "@portal/solid-ui/form/Textarea";

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
    regenerate: boolean;
    context: ChatQueryContext;
    isNewThread: boolean;
  }) => void;
  onNewThread: () => void;
  onFocus?: () => void;
  autoFocus?: boolean;
  showContextBreadcrumb?: boolean;
  disableContextEdit?: boolean;
  context: ChatQueryContext;
}) => {
  const [getMessage, setMessage] = createSignal("");

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

  const submitForm = () => {
    props.sendNewMessage({
      id: uniqueId(19),
      threadId: props.threadId || uniqueId(19),
      message: {
        content: getMessage(),
      },
      regenerate: false,
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
    <div class="chat-box min-w-[200px] max-w-[750px] space-y-1 rounded-md">
      <Show when={Boolean(props.threadId)}>
        <div class="flex px-2 pt-2 flex-row text-accent-11">
          <div class="new-chat flex pr-2 text-xs font-normal text-brand-12/80 border border-brand-12/50 rounded align-middle cursor-pointer select-none bg-white shadow-2xl">
            <HiOutlinePlus size="20px" class="py-1" />
            <div class="leading-5" onClick={props.onNewThread}>
              New thread
            </div>
          </div>
        </div>
      </Show>
      <Show when={props.showContextBreadcrumb && props.context?.length! > 0}>
        <SelectChatContext
          context={props.context[0]}
          disableContextEdit={props.disableContextEdit}
        />
      </Show>
      <div class="relative px-2 py-2 rounded-lg bg-gray-50 border border-gray-200 shadow-sm">
        <form
          class="p-0 m-0"
          onSubmit={(e) => {
            submitForm();
            e.preventDefault();
          }}
        >
          <textarea
            ref={(node) => {
              adjustTextareaHeight(node, getMessage);
              textareaRef = node;
            }}
            placeholder="Send a message"
            class="w-full max-h-[180px] px-2 text-sm text-gray-800 bg-transparent outline-none focus:outline-none resize-none placeholder:text-gray-500"
            style={{
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
          <div class="absolute bottom-2 right-0 px-2">
            <button
              class="p-1  rounded outline-none"
              classList={{
                "text-white bg-indigo-500": getMessage().trim().length > 0,
                "text-gray-500 bg-indigo-200": getMessage().trim().length == 0,
              }}
            >
              <HiOutlinePaperAirplane size="14px" />
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

const SelectChatContext = (props: {
  context: ChatQueryContext[0];
  disableContextEdit?: boolean;
}) => {
  const visibleBreadCrumbs = createMemo(() => {
    const context = props.context;
    return context.breadcrumbs.slice(props.context.breadcrumbs.length - 2);
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
