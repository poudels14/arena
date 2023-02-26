import { MetaProvider } from "@solidjs/meta";
import { PageEvent } from "../server/event";
import { createContext } from "solid-js";

// Note(sagar): this is aliased to $PROJECT/src/root.tsx
// @ts-ignore
import Root from "~/root";

const ServerContext = createContext({});
const ServerContextProvider = ServerContext.Provider;

const ServerRoot = (props: { event: PageEvent }) => {
  return (
    <ServerContextProvider value={props.event}>
      <MetaProvider tags={props.event.tags}>
        {/* TODO(sagar): <StartRouter
            url={path}
            out={event.routerContext}
            location={path}
            prevLocation={event.prevUrl}
            data={dataFn}
            routes={fileRoutes}
          >
            {docType as unknown as any}
            <Root />
          </StartRouter> */}
        <Root />
      </MetaProvider>
    </ServerContextProvider>
  );
};

export { ServerRoot, ServerContextProvider, ServerContext };
