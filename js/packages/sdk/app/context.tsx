import urlJoin from "url-join";
import { createContext, useContext } from "solid-js";

type AppContext = {
  useApiRoute: (path: string) => string;
  router: Router;
};

type Router = {
  query<T>(path: string): Promise<T>;
};

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

  const router = {
    async query(path: string) {
      const data = await fetch(urlJoin(props.urlPrefix || "", path));
      return await data.json();
    },
  };

  return (
    <AppContext.Provider value={{ useApiRoute, router }}>
      {props.children}
    </AppContext.Provider>
  );
};

export { useAppContext, AppContextProvider };
