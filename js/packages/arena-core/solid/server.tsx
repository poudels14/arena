import { MetaProvider } from "@solidjs/meta";
import { PageEvent } from "../server/event";
import { Router } from "@solidjs/router";
import { ssr } from "solid-js/web";
// Note(sagar): this is aliased to $PROJECT/src/root.tsx
// @ts-ignore
import Root from "~/root";
import { ServerContextProvider } from "./context";
import env from "../env";

const noSSR = !env.ARENA_SSR;
const docType = ssr("<!DOCTYPE html>");
const ServerRoot = <E extends PageEvent>({
  event,
  ...props
}: { event: E } & Record<string, any>) => {
  const path = event.ctx.path + event.ctx.search;

  return (
    <ServerContextProvider value={{ event, ...props }}>
      <MetaProvider tags={event.tags}>
        {noSSR ? (
          <Root />
        ) : (
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
        )}
      </MetaProvider>
    </ServerContextProvider>
  );
};

export { ServerRoot };
