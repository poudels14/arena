import { MetaProvider } from "@solidjs/meta";
import { PageEvent } from "../server/event";
import { createContext } from "solid-js";
import { Router } from "@solidjs/router";
import { ssr } from "solid-js/web";
// Note(sagar): this is aliased to $PROJECT/src/root.tsx
// @ts-ignore
import Root from "~/root";

const ServerContext = createContext({});
const ServerContextProvider = ServerContext.Provider;

const noSSR = !Arena.env.ARENA_SSR;
const docType = ssr("<!DOCTYPE html>");
const ServerRoot = ({ event }: { event: PageEvent }) => {
  const path = event.ctx.path + event.ctx.search;

  return (
    <ServerContextProvider value={event}>
      {noSSR ? (
        <Root />
      ) : (
        <MetaProvider tags={event.tags}>
          <Router
            url={path}
            // TODO(sagar)
            // out={event.routerContext}
            out={{}}
            // location={path}
            // prevLocation={event.prevUrl}
            data={undefined}
            // routes={[]}
            // routes={fileRoutes}
          >
            {docType as unknown as any}
            <Root />
          </Router>
        </MetaProvider>
      )}
    </ServerContextProvider>
  );
};

export { ServerRoot, ServerContextProvider, ServerContext };
