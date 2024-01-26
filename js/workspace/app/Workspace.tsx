import { Show, lazy, createSelector, createMemo } from "solid-js";
import { SidebarTab } from "@portal/solid-ui/sidebar";
import { Sidebar as PortalSidebar } from "@portal/solid-ui/sidebar";
import { Router, Route, useNavigate, useMatcher } from "@portal/solid-router";
import { HiOutlineDocumentChartBar, HiOutlineHome } from "solid-icons/hi";
import { QueryContextProvider } from "@portal/solid-query";
import { WorkspaceContextProvider, useWorkspaceContext } from "./context.tsx";

const Home = lazy(() => import("./home/index.tsx"));
const Documents = lazy(() => import("./documents/index.tsx"));
const AtlasAI = lazy(() => import("@portal-apps/assistant/app"));

const Workspace = () => {
  return (
    <div class="h-screen flex flex-row">
      <Router>
        <QueryContextProvider urlPrefix="/">
          <WorkspaceContextProvider>
            <WorkspaceSidebar />
            <WorkspaceRouter />
          </WorkspaceContextProvider>
        </QueryContextProvider>
      </Router>
    </div>
  );
};

const WorkspaceRouter = () => {
  const { activeWorkspace } = useWorkspaceContext();
  const atlasAi = createMemo(() => {
    const apps = activeWorkspace.apps();
    return apps.find((app) => app.slug == "atlas_ai");
  });
  return (
    <main class="content flex-1">
      <Route path="/documents">
        <Documents />
      </Route>
      <Route path="/chat">
        <Show when={atlasAi()}>
          <QueryContextProvider urlPrefix={`/w/apps/${atlasAi()!.id}/api/`}>
            <AtlasAI />
          </QueryContextProvider>
        </Show>
      </Route>
      <Route path="/" exact>
        <Home />
      </Route>
    </main>
  );
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
        active={isTab("documents")}
        onClick={() => {
          navigate("/documents");
        }}
      >
        <div>Documents</div>
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
