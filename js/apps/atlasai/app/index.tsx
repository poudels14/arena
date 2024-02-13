import { For, createMemo, createSelector, lazy, useContext } from "solid-js";
import { Route, useNavigate, useMatcher } from "@portal/solid-router";
import { Sidebar as PortalSidebar, SidebarTab } from "@portal/solid-ui/sidebar";
import {
  HiOutlineCog6Tooth,
  HiOutlineChatBubbleBottomCenter,
} from "solid-icons/hi";
import { ChatContext, ChatContextProvider } from "./chat/ChatContext.tsx";

const Chat = lazy(() => import("./chat/index.tsx"));
const Settings = lazy(() => import("./settings/index.tsx"));

const Assistant = () => {
  const navigate = useNavigate();
  const threadIdMatcher = useMatcher("/t/:threadId");
  return (
    <ChatContextProvider
      activeThreadId={threadIdMatcher()?.params?.threadId}
      onThreadReady={(threadId) => {
        navigate(`/t/${threadId}`);
      }}
    >
      <div class="w-full h-screen flex flex-row">
        <Sidebar />
        <Route path="/settings">
          <Settings />
        </Route>
        <Route
          path="/*"
          component={() => {
            return <Chat />;
          }}
        />
      </div>
    </ChatContextProvider>
  );
};

const Sidebar = () => {
  const navigate = useNavigate();
  const tabMatcher = useMatcher("/:tab/*");
  const isTabActive = createSelector(() => tabMatcher()?.params?.tab || "chat");
  const { state } = useContext(ChatContext)!;

  const threadIds = createMemo(() => {
    const threads = Object.values(state.threadsById() || {});
    if (threads) {
      threads.sort(
        (a, b) =>
          new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
      );
      const sortedThreadIds = threads.map((t) => t.id);
      return sortedThreadIds;
    }
    return [];
  });

  const isThreadActive = createSelector(() => state.activeThreadId());
  return (
    <PortalSidebar class="basis-[225px] shrink-0 no-scrollbar py-4 h-screen shadow text-sm tab:py-1 tab:px-4 tab:py-2 tab:text-gray-600 tab:text-xs tab-hover:text-gray-700 tab-active:text-black tab-active:font-medium icon:w-4 icon:h-4 icon:text-gray-400 overflow-y-auto">
      <SidebarTab
        icon={{
          svg: <HiOutlineCog6Tooth />,
        }}
        active={isTabActive("settings")}
        onClick={() => navigate("/settings")}
      >
        <div>Settings</div>
      </SidebarTab>
      <SidebarTab
        icon={{
          svg: <HiOutlineChatBubbleBottomCenter />,
        }}
        active={isTabActive("chat") || isTabActive("t")}
        onClick={() => navigate("/")}
      >
        <div>Chat</div>
      </SidebarTab>
      <PortalSidebar class="tab:pl-6 tab:py-2 tab-active:bg-gray-100 tab-hover:bg-gray-100">
        <For each={threadIds()}>
          {(threadId) => {
            return (
              <SidebarTab
                active={isThreadActive(threadId)}
                onClick={() => navigate(`/t/${threadId}`)}
              >
                <div class="overflow-hidden text-ellipsis text-nowrap">
                  {state.threadsById[threadId].title() || "Untitled"}
                </div>
              </SidebarTab>
            );
          }}
        </For>
      </PortalSidebar>
    </PortalSidebar>
  );
};

export default Assistant;
