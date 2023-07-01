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
      queryWidgetData={async (input: any) => {
        return await fetch(
          `/w/${input.appId}/widgets/${input.widgetId}/api/${
            input.field
          }?updatedAt=${input.updatedAt}&props=${encodeURI(
            JSON.stringify(input.props)
          )}`,
          {
            method: "GET",
          }
        ).then(async (r) => {
          if (r.status != 200) {
            throw new Error(await r.text());
          }
          return await r.json();
        });
      }}
    >
      <Editor appId={props.id} />
    </ApiContextProvider>
  );
};

export default App;
