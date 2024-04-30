import { Router, Route, useNavigate, useMatcher } from "@portal/solid-router";
import { Sidebar as PortalSidebar, SidebarTab } from "@portal/solid-ui/sidebar";
import { Match, Switch, createEffect, createSelector, lazy } from "solid-js";

const AgentWorkflows = lazy(() => import("./Workflows"));
const WorkflowTemplates = lazy(() => import("./workflows/templates"));

const Agents = () => {
  const matcher = useMatcher(() => "/:tab/*");
  const isTabActive = createSelector(() => matcher()?.params?.tab || "home");
  return (
    <div>
      <div>
        <Tabs isTabActive={isTabActive} />
      </div>
      <div class="py-4 px-4">
        <Route
          path="/templates"
          component={() => {
            return <WorkflowTemplates />;
          }}
        />
        <Route
          path="/"
          component={() => {
            return <AgentWorkflows />;
          }}
        />
      </div>
    </div>
  );
};

const Tabs = (props: { isTabActive: (tab: string) => boolean }) => {
  const navigate = useNavigate();
  return (
    <div>
      <PortalSidebar class="py-1 px-6 w-full text-sm flex flex-row space-x-5 justify-center border-b border-gray-200 tab:px-6 tab:py-1 tab:rounded tab:text-gray-600 tab-hover:text-gray-700 tab-hover:bg-gray-50 tab-active:bg-gray-100 tab-active:text-black">
        <SidebarTab
          // icon={{
          //   svg: <HiOutlineHome />,
          // }}
          active={props.isTabActive("home")}
          onClick={() => {
            navigate("/");
          }}
        >
          <div>Workflows</div>
        </SidebarTab>
        <SidebarTab
          // icon={{
          //   svg: <HiOutlineDocumentChartBar />,
          // }}
          active={props.isTabActive("templates")}
          onClick={() => {
            navigate("/templates");
          }}
        >
          <div>Templates</div>
        </SidebarTab>
        <SidebarTab
          // icon={{
          //   svg: <HiOutlineDocumentChartBar />,
          // }}
          active={props.isTabActive("profiles")}
          onClick={() => {
            navigate("/profiles");
          }}
        >
          <div>Actions</div>
        </SidebarTab>
      </PortalSidebar>
    </div>
  );
};

export default Agents;
