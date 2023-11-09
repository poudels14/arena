import { Show, createResource, useContext } from "solid-js";
import "highlight.js/styles/atom-one-dark";
import { SlidingDrawer } from "@arena/components/SlidingDrawer";
import { ChatContext } from "./ChatContext";

const DocumentViewer = (props: { document: any; onClose: () => void }) => {
  const { router } = useContext(ChatContext)!;

  const [document] = createResource(
    () => props.document,
    async (doc) => {
      return await router.get(`/api/documents/${doc.id}`).then((r) => r.data);
    }
  );

  return (
    <SlidingDrawer
      onClose={() => props.onClose()}
      contentClass="text-sm text-accent-12/80 overflow-y-auto"
    >
      {/* TODO(sagar): show loading UI */}
      <Show when={!document.error && document()}>
        <div class="px-5 py-3 text-lg font-medium text-accent-12 bg-brand-3">
          {document().name}
        </div>
        <div
          innerHTML={document().html}
          class="px-5 py-3 overflow-auto"
          style={"--scale-factor: 1;"}
        ></div>
      </Show>
      <Show when={document.error}>
        <div class="py-10 text-lg text-center text-red-700">
          Error loading document
        </div>
      </Show>
    </SlidingDrawer>
  );
};

export { DocumentViewer };
