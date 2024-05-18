import { AIChat, createChatStore } from "@portal/cortex/react/chat";
import { jsonStreamToAsyncIterator } from "@portal/cortex/stream/stream";
import { ErrorBoundary } from "react-error-boundary";

const Chat = (props: { agentId: string }) => {
  const store = createChatStore(
    {
      messages: {},
      async sendNewMessage(input, state) {
        const res = await fetch(`/api/chat/sendMessage`, {
          method: "POST",
          body: JSON.stringify({
            ...input,
            agentId: props.agentId,
          }),
          headers: {
            "content-type": "application/json",
          },
        });
        const iterator = jsonStreamToAsyncIterator(res.body!);
        async function* asyncGenerator() {
          for await (const { json } of iterator) {
            yield json;
          }
        }

        return asyncGenerator();
      },
    },
    {
      persistKey: "portal-cortex-demo-chat",
    }
  );

  return (
    <div className="h-full flex">
      <ErrorBoundary fallback={<div>ERROR!</div>}>
        <AIChat store={store} />
      </ErrorBoundary>
      <div className="absolute top-4 right-4">
        <div
          className="bg-gray-200 rounded px-4 py-1 cursor-pointer"
          onClick={() => {
            store.persist.clearStorage();
          }}
        >
          Reset chat
        </div>
      </div>
    </div>
  );
};

export default Chat;
