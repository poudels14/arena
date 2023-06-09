import { Routes as SolidRoutes, Route, useParams } from "@solidjs/router";
import { lazy } from "solid-js";
import { DashboardContextProvider } from "~/context";
//@ts-ignore
const App = lazy(() => import("./apps/App.tsx"));
//@ts-ignore
const Dashboard = lazy(() => import("./dashboard.tsx"));

const Routes = () => {
  return (
    <DashboardContextProvider workspaceId="1">
      <SolidRoutes>
        <Route
          path="/apps/:id"
          component={() => {
            const params = useParams();
            return <App id={params.id} />;
          }}
        />
        <Route path="/*" component={Dashboard} />
      </SolidRoutes>
    </DashboardContextProvider>
  );
};

export { Routes };
