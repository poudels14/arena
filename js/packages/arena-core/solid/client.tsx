import { MetaProvider } from "@solidjs/meta";
import { Router, RouterProps } from "@solidjs/router";
// @ts-ignore
import Root from "~/root";
import { ServerContextProvider } from "./context";

const ArenaRouter = (props: RouterProps) => {
  return (
    <Router {...props}>
      <Root />
    </Router>
  );
};

const ClientRoot = () => {
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
        >
          {/* <Root /> //Note(sagar): idk why putting Root here crashes fontend */}
        </ArenaRouter>
      </MetaProvider>
    </ServerContextProvider>
  );
};

export { ClientRoot };
export { mount } from "./mount";
