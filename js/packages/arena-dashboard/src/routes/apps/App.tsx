import { Editor } from "@arena/studio";
import { useDashboardContext } from "~/context";

const App = (props: { id: string }) => {
  const { client } = useDashboardContext();

  return (
    <Editor
      appId={props.id}
      fetchApp={(id: string) => client.apps.getApp.query(id)}
      addWidget={(widget: any) => client.widgets.addWidget.mutate(widget)}
      updateWidget={(widget: any) => client.widgets.updateWidget.mutate(widget)}
    />
  );
};

export default App;
