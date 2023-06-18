import { createContext, useContext } from "solid-js";
import { createTRPCProxyClient, httpLink } from "@trpc/client";
import type { AppRouter } from "~/api";

const DashboardContext = createContext<{
  client: ReturnType<typeof createTRPCProxyClient<AppRouter>>;
  workspaceId: string;
  user: any;
}>();

const useDashboardContext = () => useContext(DashboardContext)!;

const DashboardContextProvider = (props: {
  workspaceId: string;
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
      value={{ client, workspaceId: props.workspaceId, user: props.user }}
    >
      {props.children}
    </DashboardContext.Provider>
  );
};

export { useDashboardContext, DashboardContextProvider };
