import { Routes as SolidRoutes, Route } from "@solidjs/router";
import { lazy } from "solid-js";
//@ts-ignore
const Apps = lazy(() => import("./apps/index.tsx"));

const Routes = () => {
  return (
    <div>
      <SolidRoutes>
        <Route path="/apps" component={Apps} />
      </SolidRoutes>
    </div>
  );
};

export { Routes };
