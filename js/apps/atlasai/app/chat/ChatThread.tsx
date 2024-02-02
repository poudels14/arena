import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  lazy,
  useContext,
} from "solid-js";
import { Markdown } from "@portal/solid-ui/markdown";
import { Marked } from "marked";
import dlv from "dlv";
import deepEqual from "fast-deep-equal/es6";
import hljs from "highlight.js/lib/core";
import "highlight.js/styles/atom-one-dark.css";
import jsGrammar from "highlight.js/lib/languages/javascript";
import cssGrammar from "highlight.js/lib/languages/css";
import xmlGrammar from "highlight.js/lib/languages/xml";
import pythonGrammar from "highlight.js/lib/languages/python";
import rustGrammar from "highlight.js/lib/languages/rust";

import { EmptyThread } from "./EmptyThread";
import { ChatContext, ChatState } from "./ChatContext";
import { PluginWorkflow } from "./PluginWorkflow";
import { Chat, Document } from "../types";
import { Store } from "@portal/solid-store";
import { createQuery } from "@portal/solid-query";
import { WigetContainer } from "./Widget";

hljs.registerLanguage("javascript", jsGrammar);
hljs.registerLanguage("css", cssGrammar);
hljs.registerLanguage("html", xmlGrammar);
hljs.registerLanguage("xml", xmlGrammar);
hljs.registerLanguage("python", pythonGrammar);
hljs.registerLanguage("rust", rustGrammar);

const marked = new Marked({});

const ChatThread = (props: { showDocument(doc: any): void }) => {
  let chatMessagesContainerRef: any;
  let chatMessagesRef: any;
  const { state, sendNewMessage, activeChatThread } = useContext(ChatContext)!;

  const sortedMessageIds = createMemo(() => {
    const messages = Object.values(activeChatThread.messages());
    messages.sort(
      (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
    );
    return messages.map((m) => m.id);
  });

  const threadTaskCallIds = createMemo(
    () => {
      const messages = Object.values(activeChatThread.messages());
      const taskIds = messages
        .map((message) => dlv(message, ["message", "tool_calls", 0, "id"]))
        .filter((id) => Boolean(id));
      return taskIds;
    },
    [],
    {
      equals(prev, next) {
        return deepEqual(prev, next);
      },
    }
  );

  const threadTaskExecutionsById = createQuery<Chat.TaskExecution[]>(() => {
    const activeThreadId = state.activeThreadId();
    if (!activeThreadId) return null;
    // reload tasks if the ids change
    void threadTaskCallIds();
    return `/chat/threads/${activeThreadId}/tasks`;
  }, {});

  const error = createMemo<{ message: string } | null>(() => {
    // TODO
    // const activeThreadId = state.activeThreadId();
    // const errors = state.errors();
    // return errors.find((e) => e.threadId == activeThreadId);
    return null;
  });

  return (
    <div
      ref={chatMessagesContainerRef}
      class="flex justify-center h-full overflow-y-auto scroll:w-1 thumb:rounded thumb:bg-gray-400"
    >
      <div class="px-4 flex-1 min-w-[350px] max-w-[650px]">
        <Show when={!state.activeThreadId()}>
          <EmptyThread />
        </Show>
        <div
          ref={chatMessagesRef}
          class="chat-messages pt-2 pb-28 text-sm text-accent-12/80 space-y-1"
        >
          <For each={sortedMessageIds()}>
            {(messageId, index) => {
              // Note(sagar): use state directly to only update message
              // content element when streaming
              const message = activeChatThread.messages[messageId]!;
              if (index() == sortedMessageIds().length - 1) {
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
                <Switch>
                  <Match when={message.metadata.workflow!()}>
                    <PluginWorkflow id={message.metadata.workflow!.id()!} />
                  </Match>
                  <Match when={message.message()}>
                    <ChatMessage
                      state={state}
                      message={message}
                      task={
                        threadTaskExecutionsById.data[
                          message.message.tool_calls[0].id() as any as number
                        ]
                      }
                      showDocument={props.showDocument}
                    />
                  </Match>
                </Switch>
              );
            }}
          </For>
          <Show when={sendNewMessage.isPending && !sendNewMessage.isIdle}>
            <div>
              <ChatMessage
                state={state}
                // @ts-expect-error
                message={sendNewMessage.input}
                showDocument={props.showDocument}
              />
            </div>
          </Show>
        </div>
        <Show when={error()}>
          <div class="py-4 text-center bg-red-50 text-red-700">
            {error()?.message}
          </div>
        </Show>
      </div>
    </div>
  );
};

const ChatMessage = (props: {
  state: ChatState;
  message: Store<Pick<Chat.Message, "id" | "message" | "metadata" | "role">>;
  task?: Store<Chat.TaskExecution | undefined>;
  showDocument(doc: any): void;
}) => {
  const tokens = createMemo(() => {
    const content = props.message.message.content!();
    if (content) {
      return marked.lexer(content);
    }
    return null;
  });
  const uniqueDocuments = createMemo(() => {
    // TODO:
    // const allDocs = props.state.documents() || [];
    const allDocs = [] as Document[];
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

  const role = () => props.message.role() || "user";
  return (
    <div class="flex flex-row w-full space-x-5">
      <div
        class="mt-4 w-8 h-8 text-[0.6rem] font-medium leading-8 rounded-xl border select-none text-center text-gray-600"
        classList={{
          "bg-[hsl(60_28%_95%)]": role() == "user",
          "bg-brand-3": role() == "ai",
        }}
      >
        {role() == "ai" ? "AI" : "User"}
      </div>
      <div class="flex-1 space-y-2" data-message-id={props.message.id()}>
        <div
          class="message px-4 py-1 rounded-lg leading-6"
          classList={{
            "bg-[hsl(60_28%_95%)]": role() == "user",
            "text-gray-800": role() == "ai",
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
                      class="block my-2 px-4 py-4 rounded bg-gray-800 text-white whitespace-pre overflow-auto"
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
        <Show when={props.task && props.task()}>
          {/* @ts-expect-error */}
          <TaskExecution task={props.task!} />
        </Show>
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

const Timer = lazy(() => import("../../extensions/clock/Timer"));
const TaskExecution = (props: { task: Store<Chat.TaskExecution> }) => {
  return (
    <Switch>
      <Match when={props.task.taskId() == "start_timer"}>
        <WigetContainer Widget={Timer} state={props.task.state()} />
      </Match>
      <Match when={true}>
        <div>Unsupported task</div>
      </Match>
    </Switch>
  );
};

export { ChatThread };
