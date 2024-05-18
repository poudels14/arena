import { LoaderFunctionArgs, json } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";

import { ClientOnly } from "./client-only";
import AIChat from "./chat.client";

export async function loader(args: LoaderFunctionArgs) {
  return json({
    agentId: args.params.agent!,
  });
}

const Chat = () => {
  const data = useLoaderData<typeof loader>();
  return (
    <div className="h-screen">
      <ClientOnly fallback={<></>}>
        {() => <AIChat agentId={data.agentId} />}
      </ClientOnly>
    </div>
  );
};

export default Chat;
