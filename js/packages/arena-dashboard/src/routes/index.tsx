import { Routes as SolidRoutes, Route, useParams } from "@solidjs/router";
import { lazy } from "solid-js";
import { DashboardContextProvider } from "~/context";

const App = lazy(() => import("./apps/App.tsx"));
const Dashboard = lazy(() => import("./dashboard.tsx"));
const Waitlisted = lazy(() => import("./waitlist.tsx"));

const Routes = (props: { user: any }) => {
  return (
    <DashboardContextProvider workspaceId="1" user={props.user}>
      <SolidRoutes>
        <Route
          path="/apps/:id"
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
