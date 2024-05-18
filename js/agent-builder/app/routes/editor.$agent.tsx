import { json } from "@remix-run/node";
import { LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";

import { AgentEditor } from "../editor";
import { useAgentNodeContext } from "../editor/AgentNodes";
import { trpc } from "../trpc";

export async function loader({ params }: LoaderFunctionArgs) {
  return json({ id: params.agent! });
}

export default function AgentEditorContainer() {
  const { id: agentId } = useLoaderData<typeof loader>();
  const agent = trpc.getAgent.useQuery({
    id: agentId,
  });
  const updateAgent = trpc.updateAgent.useMutation();

  const { nodes: agentNodes } = useAgentNodeContext();
  if (!agent.data) {
    return null;
  }
  return (
    <AgentEditor
      agentNodes={agentNodes}
      graph={agent.data.graph}
      saveGraph={async (graph) => {
        await updateAgent.mutate({
          id: agentId,
          graph,
        });
        return true;
      }}
    />
  );
}
