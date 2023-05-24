import { createContext, splitProps, useContext } from "solid-js";
import { App } from "./types/app";
import { Widget } from "@arena/widgets/schema";

type ApiRoutes = {
  fetchApp: (appId: string) => Promise<App>;
  addWidget: (widget: {
    id: string;
    appId: string;
    description?: string;
    parentId: string;
    templateId: string;
    position: {
      after: string | null;
      before: string | null;
    };
  }) => Promise<Widget>;
  updateWidget: (
    widget: { id: string } & Partial<Omit<Widget, "id" | "template">>
  ) => Promise<Widget>;
  queryWidgetData: () => Promise<any>;
};

const ApiContext = createContext<{ routes: ApiRoutes }>();
function useApiContext() {
  return useContext(ApiContext)!;
}

const ApiContextProvider = (props: ApiRoutes & { children: any }) => {
  let [_, routes] = splitProps(props, ["children"]);
  return (
    <ApiContext.Provider value={{ routes }}>
      {props.children}
    </ApiContext.Provider>
  );
};

export { useApiContext, ApiContextProvider };
export type { ApiRoutes };
