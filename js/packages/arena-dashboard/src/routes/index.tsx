import { Routes as SolidRoutes, Route } from "@solidjs/router";
import { lazy } from "solid-js";
import { Dashboard } from "./dashboard";
//@ts-ignore
const App = lazy(() => import("./apps/App.tsx"));

const Routes = () => {
  return (
    <div>
      <SolidRoutes>
        <Route path="/apps/:id" component={App} />
        <Route path="/*" component={Dashboard} />
      </SolidRoutes>
    </div>
  );
};

export { Routes };
