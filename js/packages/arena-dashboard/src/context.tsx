import { createContext, useContext } from "solid-js";
import { createTRPCProxyClient, httpLink } from "@trpc/client";
import type { AppRouter } from "~/api";

const DashboardContext = createContext<{
  client: ReturnType<typeof createTRPCProxyClient<AppRouter>>;
  workspaceId: string;
}>();

const useDashboardContext = () => useContext(DashboardContext)!;

const DashboardContextProvider = (props: {
  workspaceId: string;
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
      value={{ client, workspaceId: props.workspaceId }}
    >
      {props.children}
    </DashboardContext.Provider>
  );
};

export { useDashboardContext, DashboardContextProvider };
