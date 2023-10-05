import {
  Switch,
  Match,
  For,
  Show,
  createComputed,
  createEffect,
  createMemo,
  createSignal,
  useContext,
} from "solid-js";
import { InlineIcon } from "@arena/components";
import { Markdown } from "@arena/components/markdown";
import { Marked } from "marked";
import hljs from "highlight.js/lib/core";
import "highlight.js/styles/atom-one-dark";
import jsGrammar from "highlight.js/lib/languages/javascript";
import cssGrammar from "highlight.js/lib/languages/css";
import xmlGrammar from "highlight.js/lib/languages/xml";
import pythonGrammar from "highlight.js/lib/languages/python";
import rustGrammar from "highlight.js/lib/languages/rust";
import SendIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/send-message";
import AddIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/plus";
import { ChatContext, ChatState } from "./ChatContext";
import { Chat } from "../types";
import { Store } from "@arena/solid-store";
import { useNavigate } from "@solidjs/router";
import { DocumentViewer } from "./DocumentViewer";
import { EmptyAIThread } from "./EmptyAIThread";

hljs.registerLanguage("javascript", jsGrammar);
hljs.registerLanguage("css", cssGrammar);
hljs.registerLanguage("html", xmlGrammar);
hljs.registerLanguage("xml", xmlGrammar);
hljs.registerLanguage("python", pythonGrammar);
hljs.registerLanguage("rust", rustGrammar);

const marked = new Marked({});

const AIChat = (props: { channelId: string; threadId: string }) => {
  const { state } = useContext(ChatContext)!;
  const navigate = useNavigate();
  let chatMessagesContainerRef: any;
  let chatMessagesRef: any;

  const messages = createMemo(() => {
    if (props.threadId) {
      const thread = state.threads[props.threadId]();
      if (!thread) {
        return null;
      }
      return thread.messages;
    }
    return null;
  });

  const error = createMemo(() => {
    const errors = state.errors();
    return errors.find(
      (e) => e.channelId == props.channelId && e.threadId == props.threadId
    );
  });

  const [drawerDocument, setDrawerDocument] = createSignal<any>(null);
  return (
    <div class="chat relative flex-1 h-full">
      <div
        ref={chatMessagesContainerRef}
        class="flex justify-center h-full overflow-y-auto"
      >
        <div class="flex-1 max-w-[650px]">
          {/* <ThreadInfo threadId={props.threadId} /> */}
          <div
            ref={chatMessagesRef}
            class="chat-messages pb-24 py-2 space-y-5 text-sm text-accent-12/80"
          >
            <Switch>
              <Match when={Boolean(props.threadId)}>
                <For each={messages()!}>
                  {(m, index) => {
                    const msgArr = messages()!;
                    // Note(sagar): use state directly to only update message
                    // content element when streaming
                    const message =
                      state.threads[props.threadId].messages[index()];
                    if (index() == msgArr.length - 1) {
                      createEffect(() => {
                        void message.message();
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

                    return (
                      <ChatMessage
                        state={state}
                        message={message}
                        showDocument={setDrawerDocument}
                      />
                    );
                  }}
                </For>
              </Match>
              <Match when={!state.activeThreadId()}>
                <EmptyAIThread />
              </Match>
            </Switch>

            <Show when={error()}>
              <div class="py-4 text-center bg-red-50 text-red-700">
                {error()?.message}
              </div>
            </Show>
          </div>
        </div>
      </div>
      <div class="chatbox-container absolute bottom-2 w-full flex justify-center pointer-events-none">
        <div class="w-8"></div>
        <div class="flex-1 max-w-[560px] rounded-lg pointer-events-auto backdrop-blur-xl">
          <div class="flex p-2 flex-row text-accent-11">
            <Show when={Boolean(props.threadId)}>
              <div class="new-chat flex pr-2 text-xs font-normal text-brand-12/80 border border-brand-12/50 rounded align-middle cursor-pointer select-none bg-white shadow-2xl">
                <InlineIcon size="20px" class="py-1">
                  <path d={AddIcon[0]} />
                </InlineIcon>
                <div
                  class="leading-5"
                  onClick={() => navigate(`/chat/${props.channelId}`)}
                >
                  New thread
                </div>
              </div>
            </Show>
          </div>
          <Chatbox
            isGeneratingResponse={state.threads[props.threadId].blocked}
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

const ThreadInfo = (props: { threadId: string }) => {
  return (
    <div class="h-10 bg-brand-10/20">
      <div>Thread info</div>
    </div>
  );
};

const ChatMessage = (props: {
  state: Store<ChatState>;
  message: Store<Chat.Message>;
  showDocument(doc: any): void;
}) => {
  const tokens = createMemo(() => {
    const content = props.message.message.content();
    if (content) {
      return marked.lexer(typeof content == "function" ? content() : content);
    }
    return null;
  });
  const uniqueDocuments = createMemo(() => {
    const allDocs = props.state.documents() || [];
    const docs = props.message.metadata.documents!() || [];
    const uniqueDocs: any[] = [];
    docs.forEach((d: any) => {
      if (!uniqueDocs.find((ud) => ud.id == d.documentId)) {
        const document = allDocs.find((ad) => ad.id == d.documentId);
        uniqueDocs.push({
          id: d.documentId,
          name: document?.name,
        });
      }
    });
    return uniqueDocs;
  });

  const role = () => props.message.role();

  return (
    <div class="flex flex-row w-full space-x-3">
      <div
        class="mt-2 w-8 h-8 text-[0.6rem] font-medium leading-8 rounded-lg border select-none text-center"
        classList={{
          "bg-blue-100": role() == "user",
          "bg-brand-3": role() == "ai",
        }}
      >
        {role() == "ai" ? "AI" : "User"}
      </div>
      <div class="flex-1 space-y-2" data-message-id={props.message.id()}>
        <div
          class="message px-3 py-1 rounded-sm"
          classList={{
            "bg-blue-100": role() == "user",
            "bg-brand-3": role() == "ai",
            "border border-red-700": props.message.streaming(),
          }}
          style={"letter-spacing: 0.1px; word-spacing: 1px"}
        >
          <Show when={tokens()}>
            <Markdown
              tokens={tokens()}
              renderer={{
                code(props) {
                  const highlighted =
                    props.lang && hljs.listLanguages().includes(props.lang);
                  return (
                    <code
                      class="block my-2 px-4 py-4 rounded bg-gray-800 text-white overflow-auto"
                      innerHTML={
                        highlighted
                          ? hljs.highlight(props.text, {
                              language: props.lang,
                            }).value
                          : props.text
                      }
                    />
                  );
                },
              }}
            />
          </Show>
        </div>
        <Show when={uniqueDocuments().length > 0}>
          <div class="matched-documents px-2 space-y-2">
            <div class="font-medium">Documents</div>
            <div class="px-2 space-y-1">
              <For each={uniqueDocuments()}>
                {(doc) => {
                  return (
                    <div
                      class=""
                      classList={{
                        "text-accent-9": !doc.name,
                      }}
                    >
                      <div class="inline px-2 py-1 bg-brand-10/10 rounded-sm">
                        {doc.name ? (
                          <span
                            class="cursor-pointer"
                            onClick={() => props.showDocument(doc)}
                          >
                            {doc.name}
                          </span>
                        ) : (
                          <span class="line-through">
                            Document has been deleted
                          </span>
                        )}
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};

const Chatbox = (props: { isGeneratingResponse(): boolean }) => {
  const { state, sendNewMessage } = useContext(ChatContext)!;
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
    void state.activeThreadId();
    textareaRef?.focus();
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

export { AIChat };
