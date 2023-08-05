import { JSX } from "solid-js";
import { MetaProvider } from "@solidjs/meta";
import { Router, RouterProps } from "@solidjs/router";
import { ServerContextProvider } from "./context";

const ArenaRouter = (props: RouterProps & { Root: () => JSX.Element }) => {
  return (
    <Router {...props}>
      <props.Root />
    </Router>
  );
};

const ClientRoot = (props: { Root: () => JSX.Element }) => {
  return (
    <ServerContextProvider
      value={{
        // TODO(sagar): mockFetchEvent
        event: null!,
      }}
    >
      <MetaProvider>
        <ArenaRouter
          data={(args: any) => {
            // TODO(sagar): dataFn
          }}
          Root={props.Root}
        >
          {/* <Root /> //Note(sagar): idk why putting Root here crashes fontend */}
        </ArenaRouter>
      </MetaProvider>
    </ServerContextProvider>
  );
};

export { ClientRoot };
export { mount } from "./mount";
