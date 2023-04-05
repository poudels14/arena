import { createComputed, createMemo, JSXElement } from "solid-js";
import { klona } from "klona";
import { Plugin } from "..";
import { ColumnDef } from "../column";

type Header = {
  id: string;
  colSpan: number;
  column: {
    def: ColumnDef;
  };
};

type HeaderGroup = {
  id: string;
  headers: Header[];
};

namespace Config {
  type BasicColumn = {
    /**
     * Accessor key
     */
    key: string;

    /**
     * Displlay title of the column
     */
    header?: string | (() => JSXElement);

    /**
     * Render row cell
     */
    cell?: (value: any) => JSXElement;
  };

  type GroupedColumn = {
    /**
     * Display title of the column
     */
    header: string | (() => JSXElement);

    columns: Column[];
  };

  // TODO(sagar): support grouped columns
  type Column = BasicColumn; // | GroupedColumn;

  export type Headers = {
    headers: Column[];
  };
}

type State = {
  withHeaders: {
    config: Config.Headers;
  };
};

type Methods = {
  getHeaderGroups: () => HeaderGroup[];
};

const withHeaders: Plugin<Config.Headers, State, Methods> = (config) => {
  return (table) => {
    const { setState, state } = table;
    setState("_plugins", "withHeaders", { config: klona(config) });

    const headerGroups = createMemo(() => {
      const config = state._plugins.withHeaders.config();
      const headerGroups: HeaderGroup[] = [
        {
          id: "0",
          headers: [],
        },
      ];

      config.headers.forEach((header) => {
        headerGroups[0].headers.push({
          id: header.key,
          colSpan: 1,
          column: {
            def: {
              key: header.key,
              header: header.header || header.key,
              cell: header.cell,
            },
          },
        });
      });

      return headerGroups;
    });

    createComputed(() => {
      const visibleColumns = headerGroups().flatMap((g) => {
        return g.headers.map((h) => h.column.def);
      });
      table.setState("_core", "visibleColumns", visibleColumns);
    });

    Object.assign(table, {
      getHeaderGroups() {
        return headerGroups();
      },
    });
  };
};

export { withHeaders };
