import { createContext, useContext } from "solid-js";
import urlJoin from "url-join";
import axios from "redaxios";

type AppContext = {
  useApiRoute: (path: string) => string;
  router: Router;
};

type Router = typeof axios;

const AppContext = createContext<AppContext>();
const useAppContext = () => useContext(AppContext)!;

type AppContextProviderProps = {
  urlPrefix?: string;
  children: any;
};

const AppContextProvider = (props: AppContextProviderProps) => {
  const useApiRoute = (path: string) => {
    return path;
  };

  const router = axios.create({
    fetch: (req: RequestInfo | URL, init?: RequestInit) => {
      if (typeof req !== "string") {
        throw new Error(
          "@arena/sdk fetch only supports URL string in the first argument"
        );
      }
      return fetch(urlJoin(props.urlPrefix || "", req as string), init);
    },
  });

  return (
    <AppContext.Provider value={{ useApiRoute, router }}>
      {props.children}
    </AppContext.Provider>
  );
};

export { useAppContext, AppContextProvider };
