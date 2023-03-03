import { createContext } from "solid-js";

const ServerContext = createContext({});
const ServerContextProvider = ServerContext.Provider;

export { ServerContextProvider, ServerContext };
