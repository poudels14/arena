import {
  Show,
  lazy,
  createSelector,
  createMemo,
  createSignal,
  createEffect,
  createComputed,
} from "solid-js";
import { SidebarTab } from "@portal/solid-ui/sidebar";
import { Sidebar as PortalSidebar } from "@portal/solid-ui/sidebar";
import {
  Router,
  Route,
  useNavigate,
  useMatcher,
  useLocation,
} from "@portal/solid-router";
import {
  HiOutlineDocumentChartBar,
  HiOutlineHome,
  HiOutlineCog6Tooth,
  HiSolidNoSymbol
} from "solid-icons/hi";
import { QueryContextProvider } from "@portal/solid-query";
import {
  SharedWorkspaceContextProvider,
  useSharedWorkspaceContext,
  SharingDialog,
} from "@portal/workspace-sdk";
import {
  ChatContextProvider,
  ChatIsland,
} from "@portal-apps/assistant/app/chat";
import { DragDropProvider, DragOverlay } from "@portal/solid-dnd";

const WorkspaceSettings = lazy(() => import("./settings/index.tsx"));
const AtlasAI = lazy(() => import("@portal-apps/assistant/app"));
const PortalDrive = lazy(() => import("@portal-apps/drive/app"));

const Workspace = () => {
  return (
    <QueryContextProvider urlPrefix="/">
      <SharedWorkspaceContextProvider>
        <div class="h-screen flex flex-row">
          <Router>
            <WorkspaceSidebar />
            <WorkspaceRouter />
          </Router>
        </div>
      </SharedWorkspaceContextProvider>
    </QueryContextProvider>
  );
};

const WorkspaceRouter = () => {
  const { activeWorkspace, } = useSharedWorkspaceContext();

  const portalDrive = createMemo(() => {
    const apps = activeWorkspace.apps();
    return apps.find((app) => app.slug == "portal_drive");
  });

  const atlasAi = createMemo(() => {
    const apps = activeWorkspace.apps();
    return apps.find((app) => app.slug == "atlas_ai");
  });

  const [getActiveThreadId, setActiveThreadId] = createSignal<
    string | undefined
  >(undefined);
  const [hideChatIsland, setHideChatIsland] = createSignal<boolean>(false, {
    equals() {
      // we want to retrigger hide every time location changes
      return false;
    },
  });
  const location = useLocation();
  createEffect(() => {
    // if url is updated, reset active thread id for ChatIsland
    void location.pathname;
    setHideChatIsland(true);
    setActiveThreadId(undefined);
  });
  return (
    <div class="relative flex-1">
      <DragDropProvider>
        <main class="content overflow-auto no-scrollbar">
          <Route path="/settings">
            <Show when={atlasAi()}>
              <CurrentAppSetter app={null} showChatIsland={false} />
              <WorkspaceSettings />
            </Show>
          </Route>
          <Route path="/files">
            <Show when={portalDrive()}>
              <CurrentAppSetter app={portalDrive()!} showChatIsland={true} />
              <QueryContextProvider urlPrefix={`/w/apps/${portalDrive()!.id}/`}>
                <PortalDrive />
              </QueryContextProvider>
            </Show>
          </Route>
          <Route path="/chat">
            <Show when={atlasAi()}>
              <CurrentAppSetter app={atlasAi()!} showChatIsland={false} />
              <QueryContextProvider urlPrefix={`/w/apps/${atlasAi()!.id}/api/`}>
                <AtlasAI />
              </QueryContextProvider>
            </Show>
          </Route>
          <Route path="/" exact>
            {() => {
              const navigate = useNavigate();
              navigate("/chat");
            }}
          </Route>
        </main>
        <QueryContextProvider urlPrefix={`/w/apps/${atlasAi()!.id}/api/`}>
          <ChatContextProvider
            activeThreadId={getActiveThreadId()}
            onThreadReady={(threadId) => {
              setActiveThreadId(threadId);
            }}
          >
            <ChatIsland
              onNewThread={() => setActiveThreadId(undefined)}
              hide={hideChatIsland()}
            />
          </ChatContextProvider>
        </QueryContextProvider>
        <DragOverlay />
      </DragDropProvider>
      <SharingDialog />
    </div>
  );
};

const CurrentAppSetter = (props: { app: any; showChatIsland: boolean }) => {
  const { setActiveApp, setChatIslandVisibility } = useSharedWorkspaceContext();
  createComputed(() => {
    setActiveApp(props.app);
    setChatIslandVisibility(props.showChatIsland);
  });
  return <></>;
};

const WorkspaceSidebar = () => {
  const { isFocusModeOn, toggleFocusMode } = useSharedWorkspaceContext();
  const navigate = useNavigate();
  const matcher = useMatcher(() => "/:tab/*");
  const isTab = createSelector(() => matcher()?.params?.tab || "home");
  const width = createMemo(() => {
    return isFocusModeOn() ? "25px" : "150px";
  })
  return (
    <div class="max-md:hidden md:block bg-slate-50">
      <PortalSidebar
        width={width()}
        class="py-4 h-[calc(100vh-theme(spacing.8))] text-sm tab:py-1.5 tab:text-gray-600 tab-hover:text-gray-700 tab-active:text-black tab-active:font-medium"
        classList={{
          "tab:justify-center": isFocusModeOn(),
          "px-5": !isFocusModeOn()
        }}
      >
        <SidebarTab
          icon={{
            svg: <HiOutlineHome />,
          }}
          active={isTab("chat")}
          onClick={() => {
            navigate("/chat");
          }}
        >
          <Show when={!isFocusModeOn()}>
            <div>Chat</div>
          </Show>
        </SidebarTab>
        <SidebarTab
          icon={{
            svg: <HiOutlineDocumentChartBar />,
          }}
          active={isTab("files")}
          onClick={() => {
            navigate("/files");
          }}
        >
          <Show when={!isFocusModeOn()}>
            <div>Files</div>
          </Show>
        </SidebarTab>
      </PortalSidebar>
      <div class="h-8 flex justify-center items-center text-center text-xs font-medium">
        <div
          class="flex px-1 py-1 cursor-pointer rounded hover:bg-gray-200"
          onClick={() => toggleFocusMode()}
        >
          <HiSolidNoSymbol size={16} />
        </div>
        <Show when={!isFocusModeOn()}>
          <div
            class="flex px-1 py-1 cursor-pointer rounded hover:bg-gray-200"
            classList={{
              "text-gray-700": !isTab("settings"),
              "text-black": isTab("settings"),
            }}
            onClick={() => {
              navigate("/settings");
            }}
          >
            <HiOutlineCog6Tooth size={16} />
          </div>
        </Show>
      </div>
    </div>
  );
};

export { Workspace };
