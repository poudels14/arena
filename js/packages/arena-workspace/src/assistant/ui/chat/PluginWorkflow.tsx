import { useContext, createEffect } from "solid-js";
import { ChatContext } from "./ChatContext";

const PluginWorkflow = (props: { id: string }) => {
  const { pluginWorkflowStream } = useContext(ChatContext)!;
  createEffect(() => {
    try {
      const stream = pluginWorkflowStream();
      if (stream) {
        (async () => {
          for await (const { json } of stream) {
            console.log("json =", json);
          }
        })();
      }
    } catch (e) {
      console.error(e);
    }
  });

  return <div>WORKFLOW!</div>;
};

export { PluginWorkflow };
