import { Match, Switch } from "solid-js";
import { Editor } from "@arena/editor";

const App = () => {
  const app = {
    mode: "edit",
    id: "app1",
    name: "My first app!",
    description: "A description for my new app",
    components: [],
  };

  return (
    <Switch>
      <Match when={app.mode === "edit"}>
        <Editor app={app} />
      </Match>
    </Switch>
  );
};

export default App;
