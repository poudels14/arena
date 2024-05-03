import {
  For,
  Match,
  Show,
  Switch,
  createEffect,
  createMemo,
  createReaction,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";
import dlv from "dlv";
import deepEqual from "fast-deep-equal/es6";
import {
  HiSolidChevronDown,
  HiSolidChevronUp,
  HiOutlinePaperClip,
  HiOutlineArrowPath,
  HiSolidChevronRight,
  HiSolidChevronLeft,
} from "solid-icons/hi";

import { EmptyThread } from "./EmptyThread";
import { ChatContext, ChatQueryContext, ChatState } from "./ChatContext";
import { Chat, Document } from "../types";
import { Store } from "@portal/solid-store";
import { createQuery, resolveFullUrl } from "@portal/solid-query";
import { Markdown } from "./Markdown";

const ChatThread = (props: {
  showDocument(doc: any): void;
  removeBottomPadding?: boolean;
  contextSelection?: ChatQueryContext;
}) => {
  let chatMessagesContainerRef: any;
  let chatMessagesRef: any;
  const {
    state,
    sortedMessageIds,
    aiMessageIdsByParentId,
    selectedMessageVersionByParentId,
    selectMessageVersion,
    sendNewMessage,
    regenerateMessage,
    getActiveChatThread,
  } = useContext(ChatContext)!;

  const threadTaskCallIds = createMemo(
    () => {
      const messages = Object.values(getActiveChatThread().messages() || []);
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

  const threadTaskExecutionsById = createQuery<Chat.TaskExecution[]>(
    () => {
      const activeThreadId = state.activeThreadId();
      if (!activeThreadId) return null;
      return `/chat/threads/${activeThreadId}/tasks`;
    },
    {},
    {
      manual: true,
    }
  );

  const threadTasks = createMemo(
    () => Object.values(threadTaskExecutionsById.data() || {}),
    [],
    {
      equals(prev, next) {
        return deepEqual(prev, next);
      },
    }
  );
  createEffect(() => {
    // reload tasks if the ids change
    const ids = threadTaskCallIds();
    if (ids.length == 0) return;
    const tasks = threadTasks();
    const pendingTasks = tasks.filter((task) => task.status == "STARTED");
    threadTaskExecutionsById.refresh();

    // it's possible pendingTasks is empty is the server hasn't added the
    // task to tasks list. So, if `ids.length < tasks tasks`, keep polling
    if (ids.length != tasks.length || pendingTasks.length > 0) {
      let interval = setInterval(() => {
        threadTaskExecutionsById.refresh();
      }, 1000);
      onCleanup(() => clearInterval(interval));
    }
  });

  const error = createMemo<{ message: string } | null>(() => {
    // TODO
    // const activeThreadId = state.activeThreadId();
    // const errors = state.errors();
    // return errors.find((e) => e.threadId == activeThreadId);
    return null;
  });

  return (
    <div class="h-full overflow-y-auto scroll:w-1 thumb:rounded thumb:bg-gray-400 space-y-6">
      <Show when={state.activeThreadId()}>
        <div class="w-full flex justify-between text-sm text-gray-700 bg-gray-50 border-b border-gray-100 overflow-hidden">
          <div class="flex max-w-[450px] pl-20 pr-4 py-2 items-center font-medium whitespace-nowrap overflow-hidden text-ellipsis">
            {getActiveChatThread().title() || "Untitled"}
          </div>
          <div class="max-md:hidden md:flex basis-40 flex-col justify-center items-center pl-2 pr-8 py-1 text-xs whitespace-nowrap space-y-0.5">
            <Show when={getActiveChatThread().metadata.model()}>
              <div class="flex space-x-2">
                <div class="">Model:</div>
                <div>{getActiveChatThread().metadata.model.name()}</div>
              </div>
            </Show>
            <Show when={getActiveChatThread().metadata.profile!()}>
              <div class="flex space-x-2">
                <div class="">Profile:</div>
                <div>{getActiveChatThread().metadata.profile!.name()}</div>
              </div>
            </Show>
          </div>
        </div>
      </Show>
      <div
        ref={chatMessagesContainerRef}
        class="flex justify-center items-center"
      >
        <div class="px-4 flex-1 min-w-[350px] max-w-[750px]">
          <Switch>
            <Match
              when={
                !state.activeThreadId() &&
                !(sendNewMessage.isPending && !sendNewMessage.isIdle)
              }
            >
              <EmptyThread contextSelection={props.contextSelection} />
            </Match>
            <Match when={getActiveChatThread()}>
              <div
                ref={chatMessagesRef}
                class="chat-messages pt-2 text-sm text-accent-12/80 space-y-3"
                classList={{
                  "pb-24": !Boolean(props.removeBottomPadding),
                }}
              >
                <For each={sortedMessageIds()}>
                  {(messageId, index) => {
                    const isLastMessage = createMemo(
                      () => index() == sortedMessageIds().length - 1
                    );
                    // Note(sagar): use state directly to only update message
                    // content element when streaming
                    const message = getActiveChatThread().messages[messageId]!;
                    if (isLastMessage()) {
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
                        {/* <Match when={message.metadata.workflow!()}>
                    <PluginWorkflow id={message.metadata.workflow!.id()!} />
                  </Match> */}
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
                            showLoadingBar={
                              sendNewMessage.isStreaming && isLastMessage()
                            }
                            showRegenerate={
                              isLastMessage() && message.role() == "ai"
                            }
                            regenerateMessage={regenerateMessage}
                            versions={
                              aiMessageIdsByParentId()[message.parentId()!] ||
                              []
                            }
                            selectedVersion={
                              selectedMessageVersionByParentId()?.[
                                message.parentId()!
                              ]
                            }
                            selectVersion={selectMessageVersion}
                          />
                        </Match>
                      </Switch>
                    );
                  }}
                </For>
                <Show
                  when={
                    sendNewMessage.isPending &&
                    !sendNewMessage.isIdle &&
                    !sendNewMessage.input.regenerate()
                  }
                >
                  <div>
                    <ChatMessage
                      state={state}
                      // @ts-expect-error
                      message={sendNewMessage.input}
                      showDocument={props.showDocument}
                      versions={[]}
                    />
                  </div>
                </Show>
              </div>
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
  );
};

const ChatMessage = (props: {
  state: ChatState;
  message: Store<
    Pick<
      Chat.Message,
      "id" | "parentId" | "message" | "metadata" | "artifacts" | "role"
    >
  >;
  task?: Store<Chat.TaskExecution | undefined>;
  showDocument(doc: any): void;
  showLoadingBar?: boolean;
  showRegenerate?: boolean;
  regenerateMessage: (opt: { id: string }) => void;
  versions: string[];
  selectedVersion?: string;
  selectVersion: (parentId: string, id: string) => void;
}) => {
  const trackLatestVersion = createReaction(() => {
    props.selectVersion(
      props.message.parentId()!,
      props.versions[props.versions.length - 1]
    );
  });
  const selectedVersionIndex = createMemo(() => {
    const idx = props.versions.findIndex((v) => props.selectedVersion == v);
    return idx < 0 ? props.versions.length - 1 : idx;
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

  const role = () => {
    const id = props.message.role() || "user";
    return {
      id,
      name: id == "ai" ? "AI" : id == "system" ? null : "User",
    };
  };

  const isPending = () => props.task && props.task.status() == "STARTED";
  return (
    <div class="chat-message flex flex-row w-full space-x-5">
      <div class="w-8">
        <Show when={role().name}>
          <div
            class="mt-2 text-[0.6rem] font-medium leading-8 rounded-xl border select-none text-center text-gray-600"
            classList={{
              "bg-[hsl(60_28%_95%)]": role().id == "user",
              "bg-brand-3": role().id == "ai",
            }}
          >
            {role().name}
          </div>
        </Show>
      </div>
      <div
        class="flex-1 space-y-2 overflow-x-hidden"
        data-message-id={props.message.id()}
      >
        <Show when={props.message.message.content!()}>
          <div
            class="message px-4 py-1 rounded-lg leading-6 select-text space-y-2"
            classList={{
              "bg-[hsl(60_28%_95%)]": role().id == "user",
              "text-gray-800": role().id == "ai",
            }}
            style={"letter-spacing: 0.1px; word-spacing: 1px"}
          >
            <Markdown markdown={props.message.message.content!()!} />
            <Show when={props.message.metadata.searchResults!()?.length! > 0}>
              <SearchResults
                searchResults={props.message.metadata.searchResults!()!}
              />
            </Show>
            <div class="flex text-xs items-center space-x-2">
              <Show when={props.versions.length > 1}>
                <div class="flex text-[10px] items-center space-x-1">
                  <HiSolidChevronLeft
                    class="cursor-pointer"
                    onClick={() => {
                      if (selectedVersionIndex() > 0) {
                        props.selectVersion(
                          props.message.parentId()!,
                          props.versions[selectedVersionIndex() - 1]
                        );
                      }
                    }}
                  />
                  <div>{selectedVersionIndex() + 1}</div>
                  <div>/</div>
                  <div>{props.versions.length}</div>
                  <HiSolidChevronRight
                    class="cursor-pointer"
                    onClick={() => {
                      if (selectedVersionIndex() < props.versions.length - 1) {
                        props.selectVersion(
                          props.message.parentId()!,
                          props.versions[selectedVersionIndex() + 1]
                        );
                      }
                    }}
                  />
                </div>
              </Show>
              <Show when={props.showRegenerate}>
                <div
                  class="cursor-pointer text-gray-400 hover:text-gray-800"
                  onClick={() => {
                    trackLatestVersion(() => props.versions);
                    props.regenerateMessage({
                      id: props.message.id()!,
                    });
                  }}
                >
                  <HiOutlineArrowPath size={13} />
                </div>
              </Show>
            </div>
            <Show when={props.message.artifacts()}>
              <For each={props.message.artifacts()}>
                {(artifact) => {
                  return <Artifact {...artifact} />;
                }}
              </For>
            </Show>
          </div>
        </Show>
        <Show when={props.task && props.task()}>
          {/* @ts-expect-error */}
          <TaskExecution task={props.task!} />
        </Show>
        <Show when={props.message.metadata.error!()}>
          <div class="py-2 text-red-700">
            <b>Error: </b>
            {props.message.metadata.error!()}
          </div>
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
        <Show when={props.showLoadingBar || isPending()}>
          <InProgressBar />
        </Show>
      </div>
    </div>
  );
};

const SearchResults = (props: { searchResults: Chat.SearchResult[] }) => {
  const [showSearchResults, setShowSearchResults] = createSignal(true);
  return (
    <div class="search-results">
      <div
        class="px-2 py-1 flex font-semibold bg-gray-100 text-gray-600 rounded cursor-pointer space-x-2"
        onClick={() => {
          setShowSearchResults((prev) => !prev);
        }}
      >
        <div class="py-1">
          <HiOutlinePaperClip size={12} />
        </div>
        <div class="flex-1">Found data related to your query</div>
        <div class="py-0.5">
          <Switch>
            <Match when={showSearchResults()}>
              <HiSolidChevronUp size={14} />
            </Match>
            <Match when={!showSearchResults()}>
              <HiSolidChevronDown size={14} />
            </Match>
          </Switch>
        </div>
      </div>
      <Show when={showSearchResults()}>
        <div class="py-1 bg-gray-100">
          <For each={props.searchResults}>
            {(result) => {
              return (
                <div class="px-8">
                  <Show when={result.files.length > 0}>
                    <div class="text-gray-700">Files</div>
                    <div class="ml-4">
                      <ul class="list-disc text-gray-600">
                        <For each={result.files}>
                          {(file) => {
                            return (
                              <li class="underline cursor-pointer">
                                {file.name}
                              </li>
                            );
                          }}
                        </For>
                      </ul>
                    </div>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </Show>
    </div>
  );
};

const Artifact = (props: Chat.Artifact) => {
  return (
    <div>
      <Switch>
        <Match when={props.contentType.startsWith("image/")}>
          <img
            height="80px"
            class="h-20"
            src={resolveFullUrl(`/chat/artifacts/${props.id}/content`)}
            alt={props.name}
          />
        </Match>

        <Match when={true}>
          <div class="text-xs text-red-800">
            Unsupported content: {props.contentType}
          </div>
        </Match>
      </Switch>
    </div>
  );
};

// const Timer = lazy(() => import("../../extensions/clock/Timer"));
// const CodeInterpreter = lazy(
//   () => import("../../extensions/interpreter/Interpreter")
// );
// const Table = lazy(() => import("@portal/solid-ui/table/DefaultTable"));
const TaskExecution = (props: { task: Store<Chat.TaskExecution> }) => {
  const state = createMemo(() => {
    return props.task.state();
  });

  return <div>Not supported yet</div>;

  // return (
  //   <>
  //     <Switch>
  //       <Match when={props.task.taskId() == "start_timer"}>
  //         <WigetContainer
  //           Widget={Timer}
  //           metadata={props.task.metadata()}
  //           state={state()}
  //           UI={{
  //             Markdown,
  //             Table,
  //           }}
  //         />
  //       </Match>
  //       <Match when={props.task.taskId() == "portal_code_interpreter"}>
  //         <WigetContainer
  //           Widget={CodeInterpreter}
  //           metadata={props.task.metadata()}
  //           state={state()}
  //           UI={{
  //             Markdown,
  //             Table,
  //           }}
  //         />
  //       </Match>
  //       <Match when={true}>
  //         <div>Unsupported task</div>
  //       </Match>
  //     </Switch>
  //   </>
  // );
};

const InProgressBar = () => {
  return (
    <div class="px-4 py-2 text-gray-800 space-y-1">
      <div class="flex py-0 justify-center items-center space-x-4">
        <div class="flex-[0.2] h-1 bg-gray-300 rounded animate-pulse"></div>
        <div class="h-1 basis-1"></div>
        <div class="flex-1 h-1 bg-gray-300 rounded animate-pulse"></div>
      </div>
      <div class="flex py-0 justify-center items-center space-x-4">
        <div class="flex-1 h-1 bg-gray-300 rounded animate-pulse"></div>
        <div class="h-1 basis-2"></div>
        <div class="flex-[0.3] h-1 bg-gray-300 rounded animate-pulse"></div>
      </div>
      <div class="h-1 bg-gray-300 rounded animate-pulse"></div>
    </div>
  );
};

export { ChatThread };
