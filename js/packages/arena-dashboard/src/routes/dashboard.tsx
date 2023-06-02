import {
  Routes as SolidRoutes,
  Route,
  useLocation,
  useNavigate,
} from "@solidjs/router";
import { lazy, createEffect } from "solid-js";
import { ChatBox } from "../jarvis";
import { Sidebar } from "../sidebar";
import { Resources } from "./resources";

//@ts-ignore
const Apps = lazy(() => import("./apps/index.tsx"));

const DashboardPages = () => {
  return (
    <>
      <SolidRoutes>
        <Route path="/apps" component={Apps} />
        <Route path="/resources" component={Resources} />
      </SolidRoutes>
    </>
  );
};

const Dashboard = () => {
  const location = useLocation();
  createEffect(() => {
    if (location.pathname == "/") {
      // Note(sp): for now, navigate to /apps if "/" is visited
      const navigate = useNavigate();
      navigate("/apps");
    }
  });

  return (
    <div class="flex">
      <div class="fixed w-52 flex flex-col left-0 top-0 bottom-0 text-sm">
        <Sidebar />
      </div>
      <main class="flex-1 fixed left-52 top-0 bottom-0 right-0">
        <DashboardPages />
      </main>
      {/* <div class="fixed left-0 bottom-4 right-0 flex justify-center pointer-events-none">
        <div class="w-[700px] pointer-events-auto">
          <ChatBox />
        </div>
      </div> */}
    </div>
  );
};

export { Dashboard };
