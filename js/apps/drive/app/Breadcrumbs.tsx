import { For, Show, createMemo } from "solid-js";
import { HiOutlineFolderOpen, HiOutlineChevronRight } from "solid-icons/hi";

const Breadcrumbs = (props: {
  breadcrumbs: {
    id: string;
    title: string;
  }[];
}) => {
  const visibleBreadCrumbs = createMemo(() => {
    return props.breadcrumbs.slice(props.breadcrumbs.length - 3);
  });
  return (
    <div class="flex px-2 py-2 text-base font-medium space-x-1 text-brand-12/90">
      <div class="flex overflow-hidden">
        <div class="flex rounded overflow-hidden">
          <div class="flex px-2 cursor-pointer space-x-2 rounded">
            <div class="py-1.5">
              <HiOutlineFolderOpen size={16} />
            </div>
            <div class="py-0.5 text-nowrap">Drive</div>
          </div>
          <Show when={props.breadcrumbs.length > 0}>
            <div class="py-1.5">
              <HiOutlineChevronRight size={16} />
            </div>
          </Show>
          <Show when={visibleBreadCrumbs().length < props.breadcrumbs.length}>
            <div>...</div>
            <div class="py-1.5">
              <HiOutlineChevronRight size={16} />
            </div>
          </Show>
          <For each={visibleBreadCrumbs()}>
            {(breadcrumb, index) => {
              return (
                <>
                  <div class="flex px-2 cursor-pointer space-x-1 rounded overflow-hidden text-nowrap">
                    <div class="py-0.5 overflow-hidden text-ellipsis">
                      {breadcrumb.title}
                    </div>
                  </div>
                  <Show when={index() < visibleBreadCrumbs().length - 1}>
                    <div class="py-1">
                      <HiOutlineChevronRight size={16} />
                    </div>
                  </Show>
                </>
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
};

export { Breadcrumbs };
