import { MetaProvider } from "@solidjs/meta";
import { Router, RouterProps } from "@solidjs/router";
// @ts-ignore
import Root from "~/root";
import { ServerContextProvider } from "./server";

const ArenaRouter = (props: RouterProps) => {
  return (
    <Router {...props}>
      <Root />
    </Router>
  );
};

export default () => {
  return (
    <ServerContextProvider
      value={
        {
          // TODO(sagar): mockFetchEvent
        }
      }
    >
      <MetaProvider>
        <ArenaRouter
          data={(args: any) => {
            // TODO(sagar): dataFn
          }}
        >
          <Root />
        </ArenaRouter>
      </MetaProvider>
    </ServerContextProvider>
  );
};
