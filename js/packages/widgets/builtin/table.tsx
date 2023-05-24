import { createMemo, createComputed, For } from "solid-js";
import {
  createTableWithPlugins,
  withResizableColumns,
  withHeaders,
  withPagination,
} from "@arena/components/table";
import type { Template } from "..";

const metadata: Template.Metadata<{ rows: any[] }> = {
  id: "at-table",
  name: "Table",
  description: "Table",
  data: {
    rows: {
      title: "Rows",
      dataSource: {
        type: "dynamic",
        default: {
          source: "inline",
          value: [
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
    },
  },
  class: "bg-white",
};

const Table = (props: Template.Props<{ rows: any[] }>) => {
  const createTable = createTableWithPlugins(
    withHeaders({
      headers: [
        {
          key: "id",
          header: "Id",
        },
        {
          key: "name",
          header: "Name",
        },
        {
          key: "age",
          cell: (age) => <i>{age} years old</i>,
        },
      ],
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
  const { ui } = table;

  return (
    <table
      class="ar-widget ar-table w-full table-auto border border-gray-300"
      {...props.attributes}
    >
      <thead class="ar-thead border-(b,gray-300)">
        <For each={table.getHeaderGroups()}>
          {(group) => {
            return (
              <tr class="ar-tr">
                <For each={group.headers}>
                  {(header) => (
                    <ui.Th
                      header={header}
                      class="ar-th py-2 border-l border-gray-300"
                    >
                      {header.column.def.header}
                    </ui.Th>
                  )}
                </For>
              </tr>
            );
          }}
        </For>
      </thead>
      <tbody class="ar-tbody">
        <For each={rows()}>
          {(row, i) => {
            return (
              <ui.Tr class="ar-tr border-b border-gray-100 last:border-b-gray-300 hover:cursor-pointer hover:bg-green-100">
                <For each={row.getVisibleCells()}>
                  {(cell) => {
                    return (
                      <ui.Td cell={cell} class="ar-td text-center p-1 py-2" />
                    );
                  }}
                </For>
              </ui.Tr>
            );
          }}
        </For>
      </tbody>
    </table>
  );
};

export default Table;
export { metadata };
