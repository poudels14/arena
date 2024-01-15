import { Show, createMemo } from "solid-js";
import "highlight.js/styles/atom-one-dark";
import { SlidingDrawer } from "@portal/solid-ui/SlidingDrawer";
import { createQuery } from "@portal/solid-query";

const DocumentViewer = (props: { document: any; onClose: () => void }) => {
  const document = createMemo(() => {
    const query = createQuery<any>(`/api/documents/${props.document.id}`, {});
    return {
      get status() {
        return query.status();
      },
      // @ts-expect-error
      name: query.data.name(),
      // @ts-expect-error
      html: query.data.html(),
    };
  });

  return (
    <SlidingDrawer
      onClose={() => props.onClose()}
      contentClass="text-sm text-accent-12/80 overflow-y-auto"
    >
      {/* TODO(sagar): show loading UI */}
      <Show when={document()}>
        <div class="px-5 py-3 text-lg font-medium text-accent-12 bg-brand-3">
          {document().name}
        </div>
        <div
          innerHTML={document().html}
          class="px-5 py-3 overflow-auto"
          style={"--scale-factor: 1;"}
        ></div>
      </Show>
      <Show when={document().status && document().status != 200}>
        <div class="py-10 text-lg text-center text-red-700">
          Error loading document
        </div>
      </Show>
    </SlidingDrawer>
  );
};

export { DocumentViewer };
