import { Context, createContext, useContext } from "solid-js";
import { PageEvent } from "../server/event";

type ServerContext = {
  event: PageEvent;
};

const ServerContext = createContext<ServerContext>();
function useServerContext<T>() {
  return (
    useContext(ServerContext as unknown as Context<ServerContext & T>) || {}
  );
}

const ServerContextProvider = ServerContext.Provider;

export { ServerContextProvider, useServerContext };
