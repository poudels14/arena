import { For } from "solid-js";
import { createQuery } from "@portal/solid-query";
import { HiOutlineDocument } from "solid-icons/hi";
import { Draggable } from "@portal/solid-dnd";

const Artifacts = (props: any) => {
  const artifacts = createQuery<any[]>(() => `/chat/artifacts`, {});
  return (
    <div class="py-4 text-sm">
      <For each={artifacts.data() || []}>
        {(artifact) => {
          return (
            <Draggable asChild id={artifact.id} data={{}}>
              <div class="flex px-4 py-2 items-center space-x-2 cursor-pointer even:bg-gray-50 hover:bg-gray-100">
                <HiOutlineDocument size={14} class="py-0.5" />
                <div class="flex-1">{artifact.name}</div>
                <div class="text-xs">
                  {new Date(artifact.createdAt).toDateString()}
                </div>
              </div>
            </Draggable>
          );
        }}
      </For>
    </div>
  );
};

export { Artifacts };
