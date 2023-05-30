import { Editor, ApiContextProvider, ApiRoutes } from "@arena/studio";
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
      deleteWidget={
        ((widget: any) =>
          client.widgets.delete.mutate(widget)) as ApiRoutes["deleteWidget"]
      }
      queryWidgetData={
        ((input: any) =>
          client.dataQuery.fetch.query(input)) as ApiRoutes["queryWidgetData"]
      }
    >
      <Editor appId={props.id} />
    </ApiContextProvider>
  );
};

export default App;
