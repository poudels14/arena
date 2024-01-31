import { Show } from "solid-js";
import "highlight.js/styles/atom-one-dark.css";
import { SlidingDrawer } from "@portal/solid-ui/SlidingDrawer";
import { createQuery } from "@portal/solid-query";

const DocumentViewer = (props: { document: any; onClose: () => void }) => {
  const documentQuery = createQuery<any>(
    () => `/api/documents/${props.document.id}`,
    {}
  );

  return (
    <SlidingDrawer
      onClose={() => props.onClose()}
      contentClass="text-sm text-accent-12/80 overflow-y-auto"
    >
      {/* TODO(sagar): show loading UI */}
      <Show when={documentQuery.data()}>
        <div class="px-5 py-3 text-lg font-medium text-accent-12 bg-brand-3">
          {documentQuery.data().name}
        </div>
        <div
          innerHTML={documentQuery.data().html}
          class="px-5 py-3 overflow-auto"
          style={"--scale-factor: 1;"}
        ></div>
      </Show>
      <Show when={documentQuery.status() && documentQuery.status() != 200}>
        <div class="py-10 text-lg text-center text-red-700">
          Error loading document
        </div>
      </Show>
    </SlidingDrawer>
  );
};

export { DocumentViewer };
