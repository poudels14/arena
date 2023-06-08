import { Show, For, createResource } from "solid-js";
import TrashIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/trash";
import { useDashboardContext } from "~/context";
import { InlineIcon } from "@arena/components";
// @ts-ignore
import debounce from "debounce";

const ResourcesTable = (props: { refresh: any }) => {
  const { client, workspaceId } = useDashboardContext();
  const [resources, { refetch }] = createResource(
    () => props.refresh,
    () => client.resources.list.query({ workspaceId })
  );

  const deleteResource = debounce(async (id: string) => {
    await client.resources.archive.mutate({
      id,
      workspaceId,
    });
    refetch();
  }, 200);

  return (
    <Show when={resources()}>
      <div class="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
        <div class="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
          <table class="min-w-full divide-y divide-gray-300">
            <thead>
              <tr>
                <th
                  scope="col"
                  class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-0"
                >
                  Name
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  Resource type
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  Secret
                </th>
                <th scope="col" class="relative py-3.5 pl-3 pr-4 sm:pr-0">
                  <span class="sr-only">Delete</span>
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 bg-white">
              <For each={resources()}>
                {(resource) => {
                  return (
                    <tr>
                      <Td class="pl-4 pr-3 sm:pl-0">
                        <div class="flex items-center">
                          <div class="ml-4">
                            <div class="font-medium text-accent-12">
                              {resource.name}
                            </div>
                          </div>
                        </div>
                      </Td>
                      <Td class="px-3 text-accent-11">{resource.type}</Td>
                      <Td class="px-3">
                        <span class="inline-flex items-center rounded-md bg-green-50 px-2 py-1 text-xs font-medium text-green-700 ring-1 ring-inset ring-green-600/20">
                          {resource.secret ? "Yes" : "No"}
                        </span>
                      </Td>
                      <Td class="relative pl-3 pr-4 flex justify-center sm:pr-0">
                        <button
                          type="button"
                          class="text-accent-11/80 hover:text-accent-12"
                          onClick={() => deleteResource(resource.id!)}
                        >
                          <InlineIcon size="11px">
                            <path d={TrashIcon[0]} />
                          </InlineIcon>
                        </button>
                      </Td>
                    </tr>
                  );
                }}
              </For>
            </tbody>
          </table>
          <Show when={resources()?.length == 0}>
            <div class="py-10 text-sm text-center text-accent-9">
              No resources linked yet
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};

const Td = (props: any) => {
  return (
    <td
      class="whitespace-nowrap py-3 text-sm"
      classList={{
        [props.class]: Boolean(props.class),
      }}
    >
      {props.children}
    </td>
  );
};

export { ResourcesTable };
