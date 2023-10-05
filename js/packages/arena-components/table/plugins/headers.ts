import { createComputed, createMemo, JSXElement } from "solid-js";
import { klona } from "klona";
import { Plugin } from "..";
import { Header } from "../column";

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
    header?: string | JSXElement;

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
  export type Column = BasicColumn; // | GroupedColumn;
}

type Config = {
  headers: Config.Column[];
};

type State = {
  withHeaders: {
    config: Config;
  };
};

type Methods = {
  setHeaders: (headers: Config["headers"]) => void;
  getHeaderGroups: () => HeaderGroup[];
};

const withHeaders: Plugin<Config, State, Methods> = (config) => {
  return (table) => {
    const { setState, state } = table;

    const setHeaders = (headers: Config["headers"]) => {
      setState("_plugins", "withHeaders", "config", "headers", klona(headers));
    };
    setHeaders(config.headers);

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
      setHeaders,
      getHeaderGroups() {
        return headerGroups();
      },
    });
  };
};

export { withHeaders };
