import { createContext, useContext } from "solid-js";
import { createTRPCProxyClient, httpLink } from "@trpc/client";
import type { AppRouter } from "~/api";

type Workspace = {
  id: string;
  name: string;
};

const DashboardContext = createContext<{
  client: ReturnType<typeof createTRPCProxyClient<AppRouter>>;
  workspace: Workspace;
  user: any;
}>();

const useDashboardContext = () => useContext(DashboardContext)!;

const DashboardContextProvider = (props: {
  workspace: Workspace;
  user: any;
  children: any;
}) => {
  const client = createTRPCProxyClient<AppRouter>({
    links: [
      httpLink({
        url: "http://localhost:8000/api",
        async headers() {
          return {};
        },
      }),
    ],
  });

  return (
    <DashboardContext.Provider
      value={{ client, workspace: props.workspace, user: props.user }}
    >
      {props.children}
    </DashboardContext.Provider>
  );
};

export { useDashboardContext, DashboardContextProvider };
