import { Accessor, createContext, useContext } from "solid-js";
import { createStore } from "@arena/solid-store";
import { ActiveWidget } from "./widget/Widget";
import { Widget } from "./widget";

type App = {
  id: string;

  name: string;

  description?: string;

  /**
   * Id of the root widget
   */
  widgetId: string;

  /**
   * List of all the widgets in the app
   */
  widgets: Widget[];
};

type AppContext = {
  setSelectedWidgets: (widgets: ActiveWidget[]) => void;
  getSelectedWidgets: Accessor<ActiveWidget[]>;

  getWidgetBySlug: (slug: string) => Widget | null;
};

type AppState = {
  selectedWidgets: ActiveWidget[];
};

const AppContext = createContext<AppContext>();
const useAppContext = () => useContext(AppContext)!;

const AppContextProvider = (props: any) => {
  const [state, setState] = createStore<AppState>({
    selectedWidgets: [],
  });

  const context: AppContext = {
    setSelectedWidgets(widgets) {
      setState("selectedWidgets", widgets);
    },
    getSelectedWidgets() {
      return state.selectedWidgets();
    },
    getWidgetBySlug() {
      return null;
    },
  };
  return (
    <AppContext.Provider value={context}>{props.children}</AppContext.Provider>
  );
};

export { useAppContext, AppContextProvider };
export type { App };
