import { createMemo, createComputed, For, onMount } from "solid-js";
import {
  createTableWithPlugins,
  withResizableColumns,
  withHeaders,
  withPagination,
} from "@arena/components/table";
import { InlineIcon } from "@arena/components";
import ChevronLeft from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/chevron-left";
import ChevronRight from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/chevron-right";
import type { Template } from "..";

const metadata: Template.Metadata<{ rows: any[] }> = {
  id: "at-table",
  name: "Table",
  description: "Table",
  data: {
    rows: {
      title: "Rows",
      source: "dynamic",
      default: {
        loader: "@client/json",
      },
      preview: [
        {
          id: 1,
          name: "John Doe",
          age: 49,
        },
        {
          id: 2,
          name: "Mary Jane",
          age: 28,
        },
      ],
    },
  },
  class: "bg-white art-[>.thead](bg-gray-50)",
};

const Table = (props: Template.Props<{ rows: any[] }>) => {
  const createTable = createTableWithPlugins(
    withHeaders({
      headers: Object.keys(props.data.rows[0] || {}).map((k) => {
        return {
          key: k,
        };
      }),
    }),
    withPagination({
      pageSize: 10,
    }),
    withResizableColumns({})
  );

  const table = createTable({
    data: props.data.rows,
  });

  createComputed(() => {
    // TODO(sagar): this causes the table data to be updated with
    // original data twice. figure out a way to prevent that
    table.setData(props.data.rows);
  });

  const rows = createMemo(table.state.rows);
  const { Ui } = table;

  let tbody: any;
  onMount(() => {
    // Note(sp): set the width of each <th> so that the width doesn't
    // change after first render until resized
    tbody.style.height = tbody.clientHeight + "px";
  });

  return (
    <table
      class="ar-table flex-1 h-fit table-auto border border-gray-300"
      {...props.attributes}
    >
      <thead class="thead border-(b,gray-300)">
        <For each={table.getHeaderGroups()}>
          {(group) => {
            return (
              <tr class="tr">
                <For each={group.headers}>
                  {(header) => (
                    <Ui.Th
                      header={header}
                      class="th py-2 font-semibold border-l border-gray-300"
                    >
                      {header.column.def.header}
                    </Ui.Th>
                  )}
                </For>
              </tr>
            );
          }}
        </For>
      </thead>
      <tbody class="tbody" ref={tbody}>
        <For each={rows()}>
          {(row, i) => {
            return (
              <Ui.Tr class="tr border-b border-gray-100 last:border-b-gray-300 hover:cursor-pointer hover:bg-blue-100">
                <For each={row.getVisibleCells()}>
                  {(cell) => {
                    return (
                      <Ui.Td cell={cell} class="td text-center p-1 py-2" />
                    );
                  }}
                </For>
              </Ui.Tr>
            );
          }}
        </For>

        {/* add dummy rows so that the height of the table remains constant */}
        {/* even when the last page doesn't have enough rows */}
        {/* TODO(sp): find better way to do this */}
        <For each={[...Array(table.pageSize() - rows().length)]}>
          {(r) => (
            <tr>
              <td class="py-2 opacity-0">|</td>
            </tr>
          )}
        </For>
      </tbody>
      <tfoot class="border-t border-gray-300">
        <tr>
          <th colSpan={100}>
            <div class="px-6 py-2 space-x-2 text-accent-12 text-xs font-light flex justify-end">
              <InlineIcon
                size="16px"
                class="p-0.5"
                classList={{
                  "cursor-pointer": table.hasPreviousPage(),
                  "text-accent-8": !table.hasPreviousPage(),
                }}
                onClick={() => table.previousPage()}
              >
                <path d={ChevronLeft[0]}></path>
              </InlineIcon>
              <div class="flex space-x-1 select-none">
                <div>
                  {table.currentPage()} of {table.totalPages()}
                </div>
              </div>
              <InlineIcon
                size="16px"
                class="p-0.5"
                classList={{
                  "cursor-pointer": table.hasNextPage(),
                  "text-accent-8": !table.hasNextPage(),
                }}
                onClick={() => table.nextPage()}
              >
                <path d={ChevronRight[0]}></path>
              </InlineIcon>
            </div>
          </th>
        </tr>
      </tfoot>
    </table>
  );
};

export default Table;
export { metadata };
