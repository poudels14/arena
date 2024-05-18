import { json } from "@remix-run/node";
import { useLoaderData, Outlet } from "@remix-run/react";

import { AgentNode, AgentNodeContextProvider } from "../editor/AgentNodes";
import { listNodes } from "../agent/nodes";

export async function loader() {
  const nodes: AgentNode[] = await listNodes();
  return json({
    nodes,
  });
}

const Editor = () => {
  const data = useLoaderData<typeof loader>();
  return (
    <AgentNodeContextProvider value={data}>
      <Outlet />
    </AgentNodeContextProvider>
  );
};

export default Editor;
