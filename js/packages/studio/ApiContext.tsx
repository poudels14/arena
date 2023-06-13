import { createContext, splitProps, useContext } from "solid-js";
import { App } from "./types/app";
import { Widget } from "@arena/widgets/schema";
import { MutationResponse } from "./types";

type Layout = {
  position: {
    after: string | null;
    before: string | null;
  };
};
type ApiRoutes = {
  fetchApp: (appId: string) => Promise<App>;
  addWidget: (widget: {
    id: string;
    appId: string;
    description?: string;
    parentId: string;
    templateId: string;
    config: {
      layout: Layout;
      data?: any;
    };
  }) => Promise<MutationResponse>;
  updateWidget: (
    widget: { id: string } & Partial<Omit<Widget, "id" | "template">>
  ) => Promise<MutationResponse>;
  deleteWidget: (req: {
    id: Widget["id"];
    config: {
      layout: Layout;
    };
  }) => Promise<MutationResponse>;
  queryWidgetData: (req: {
    appId: string;
    widgetId: string;
    field: string;
    updatedAt: string;
    props: any;
  }) => Promise<any>;
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
