import { createSyncedStore } from "@arena/solid-store";
import { Routes as SolidRoutes, Route, useParams } from "@solidjs/router";
import { createMemo, lazy } from "solid-js";
import { DashboardContextProvider } from "~/context";

const App = lazy(() => import("./apps/App.tsx"));
const Dashboard = lazy(() => import("./dashboard.tsx"));
const Waitlisted = lazy(() => import("./waitlist.tsx"));

const Routes = (props: { user: any }) => {
  const [state, setState] = createSyncedStore(
    {
      selectedWorkspace: null,
    },
    {
      storeKey: "dashboard/routes/index",
    }
  );

  const workspace = createMemo(() => {
    const workspaces = props.user.workspaces;
    const selected = workspaces.find(
      (w: any) => w.id == state.selectedWorkspace
    );
    if (!selected) {
      setState("selectedWorkspace", workspaces[0].id);
    }
    return selected || workspaces[0];
  });

  return (
    <DashboardContextProvider workspace={workspace()} user={props.user}>
      <SolidRoutes>
        <Route
          path="/apps/:id"
          matchFilters={{
            id: (id) => {
              return !["new"].includes(id);
            },
          }}
          component={() => {
            const params = useParams();
            return <App id={params.id} />;
          }}
        />
        <Route path="/waitlisted" component={Waitlisted} />
        <Route path="/*" component={Dashboard} />
      </SolidRoutes>
    </DashboardContextProvider>
  );
};

export default Routes;
