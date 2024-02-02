import {
  Show,
  lazy,
  createSelector,
  createMemo,
  createSignal,
  createEffect,
  createComputed,
  Setter,
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
import { HiOutlineDocumentChartBar, HiOutlineHome } from "solid-icons/hi";
import { QueryContextProvider } from "@portal/solid-query";
import { SharedWorkspaceContextProvider } from "@portal/workspace-sdk";
import {
  ChatContextProvider,
  ChatIsland,
} from "@portal-apps/assistant/app/chat";
import { WorkspaceContextProvider, useWorkspaceContext } from "./context";

const Home = lazy(() => import("./home/index.tsx"));
const AtlasAI = lazy(() => import("@portal-apps/assistant/app"));
const PortalDrive = lazy(() => import("@portal-apps/drive/app"));

const Workspace = () => {
  const [getActiveApp, setActiveApp] = createSignal(null);
  return (
    <SharedWorkspaceContextProvider activeApp={getActiveApp()}>
      <div class="h-screen flex flex-row">
        <Router>
          <QueryContextProvider urlPrefix="/">
            <WorkspaceContextProvider>
              <WorkspaceSidebar />
              <WorkspaceRouter setActiveApp={setActiveApp} />
            </WorkspaceContextProvider>
          </QueryContextProvider>
        </Router>
      </div>
    </SharedWorkspaceContextProvider>
  );
};

const WorkspaceRouter = (props: { setActiveApp: Setter<any> }) => {
  const { activeWorkspace } = useWorkspaceContext();

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
    <main class="relative content flex-1">
      <Route path="/drive">
        <Show when={portalDrive()}>
          <CurrentAppSetter
            setActiveApp={props.setActiveApp}
            app={portalDrive()!}
          />
          <QueryContextProvider urlPrefix={`/w/apps/${portalDrive()!.id}/`}>
            <PortalDrive />
          </QueryContextProvider>
        </Show>
      </Route>
      <Route path="/chat">
        <Show when={atlasAi()}>
          <CurrentAppSetter
            setActiveApp={props.setActiveApp}
            app={atlasAi()!}
          />
          <QueryContextProvider urlPrefix={`/w/apps/${atlasAi()!.id}/api/`}>
            <AtlasAI />
          </QueryContextProvider>
        </Show>
      </Route>
      <Route path="/" exact>
        <Home />
      </Route>
      <QueryContextProvider urlPrefix={`/w/apps/${atlasAi()!.id}/api/`}>
        <ChatContextProvider
          activeThreadId={getActiveThreadId()}
          singleThreadOnly={true}
          onThreadReady={(threadId) => {
            setActiveThreadId(threadId);
          }}
        >
          <ChatIsland hide={hideChatIsland()} />
        </ChatContextProvider>
      </QueryContextProvider>
    </main>
  );
};

const CurrentAppSetter = (props: { setActiveApp: Setter<any>; app: any }) => {
  createComputed(() => {
    props.setActiveApp(props.app);
  });
  return <></>;
};

const WorkspaceSidebar = () => {
  const navigate = useNavigate();
  const matcher = useMatcher("/:tab/*");
  const isTab = createSelector(() => matcher()?.params?.tab || "home");
  return (
    <PortalSidebar
      width="200px"
      class="py-4 pl-8 h-screen text-sm bg-blue-50 tab:py-1.5 tab:text-gray-600 tab-hover:text-gray-700 tab-active:text-black tab-active:font-medium"
    >
      <SidebarTab
        icon={{
          svg: <HiOutlineHome />,
        }}
        active={isTab("home")}
        onClick={() => {
          navigate("/");
        }}
      >
        <div>Home</div>
      </SidebarTab>
      <SidebarTab
        icon={{
          svg: <HiOutlineDocumentChartBar />,
        }}
        active={isTab("drive")}
        onClick={() => {
          navigate("/drive");
        }}
      >
        <div>Drive</div>
      </SidebarTab>
      <SidebarTab
        icon={{
          svg: <HiOutlineHome />,
        }}
        active={isTab("chat")}
        onClick={() => {
          navigate("/chat");
        }}
      >
        <div>Chat</div>
      </SidebarTab>
    </PortalSidebar>
  );
};

export { Workspace };
