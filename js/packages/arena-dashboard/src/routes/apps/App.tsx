import { ApiContextProvider, ApiRoutes } from "@arena/appkit";
import { Editor } from "@arena/studio";
import { useDashboardContext } from "~/context";

const App = (props: { id: string }) => {
  const { client } = useDashboardContext();
  return (
    <ApiContextProvider
      fetchApp={
        ((id: string) => client.apps.get.query(id)) as ApiRoutes["fetchApp"]
      }
      addWidget={
        ((widget: any) =>
          client.widgets.add.mutate(widget)) as ApiRoutes["addWidget"]
      }
      updateWidget={
        ((widget: any) =>
          client.widgets.update.mutate(widget)) as ApiRoutes["updateWidget"]
      }
      queryWidgetData={
        ((input: any) =>
          client.dataQuery.fetch.query({
            appId: props.id,
            workspaceId: "workspace_1",
            widgetId: "VXJEWCbrXURzaqdq46YpKp",
            field: "rows",
            params: {
              id: "1",
            },
          })) as ApiRoutes["queryWidgetData"]
      }
    >
      <Editor appId={props.id} />
    </ApiContextProvider>
  );
};

export default App;
