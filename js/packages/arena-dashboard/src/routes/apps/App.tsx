import { Editor } from "@arena/studio";
import { createTRPCProxyClient, httpLink } from "@trpc/client";
import type { AppRouter } from "~/api";

const App = (props: { id: string }) => {
  const client = createTRPCProxyClient<AppRouter>({
    links: [
      httpLink({
        url: "http://localhost:8000/api",
        async headers() {
          return {};
        },
      }),
    ],
  });

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
