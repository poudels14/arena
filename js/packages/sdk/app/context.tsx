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

const AppContextProvider = (props: any) => {
  const useApiRoute = (path: string) => {
    return path;
  };

  const router = {
    async query<T>(path: string) {
      const data = await fetch(path);
      return data.json();
      // return null as T;
    },
  };

  return (
    <AppContext.Provider value={{ useApiRoute, router }}>
      {props.children}
    </AppContext.Provider>
  );
};

export { useAppContext, AppContextProvider };
