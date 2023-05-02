import { Routes as SolidRoutes, Route, useParams } from "@solidjs/router";
import { lazy } from "solid-js";
import { Dashboard } from "./dashboard";
//@ts-ignore
const App = lazy(() => import("./apps/App.tsx"));

const Routes = () => {
  return (
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
  );
};

export { Routes };
