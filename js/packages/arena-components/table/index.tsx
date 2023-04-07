import { createStore } from "@arena/solid-store";
import { klona } from "klona";
import { batch, createComputed, JSX, splitProps } from "solid-js";
import { Header } from "./column";
import { Cell, Row } from "./row";
import { BaseConfig, InternalTable, Plugin, Table, TableState } from "./types";

function createTableWithPlugins<C, PS, M>(
  plugin1: ReturnType<Plugin<C, PS, M>>
): <R>(c: BaseConfig<R>) => Table<PS, M, R>;

function createTableWithPlugins<C1, PS1, M1, C2, PS2, M2>(
  plugin1: ReturnType<Plugin<C1, PS1, M1>>,
  plugin2: ReturnType<Plugin<C2, PS2, M2>>
): <R>(c: BaseConfig<R>) => Table<PS1 & PS2, M1 & M2, R>;

function createTableWithPlugins<C1, PS1, M1, C2, PS2, M2, C3, PS3, M3>(
  plugin1: ReturnType<Plugin<C1, PS1, M1>>,
  plugin2: ReturnType<Plugin<C2, PS2, M2>>,
  plugin3: ReturnType<Plugin<C3, PS3, M3>>
): <R>(c: BaseConfig<R>) => Table<PS1 & PS2 & PS3, M1 & M2 & M3, R>;

function createTableWithPlugins<
  C1,
  PS1,
  M1,
  C2,
  PS2,
  M2,
  C3,
  PS3,
  M3,
  C4,
  PS4,
  M4
>(
  plugin1: ReturnType<Plugin<C1, PS1, M1>>,
  plugin2: ReturnType<Plugin<C2, PS2, M2>>,
  plugin3: ReturnType<Plugin<C3, PS3, M3>>,
  plugin4: ReturnType<Plugin<C4, PS4, M4>>
): <R>(c: BaseConfig<R>) => Table<PS1 & PS2 & PS3 & PS4, M1 & M2 & M3 & M4, R>;

/**
 * Note(sagar): even though internal functions aren't exposed, internal
 * state is exposed through {@link table.state} to make it easier to extend
 * the table functionality. Internal state should be prefixed with `_` and
 * the state structure can change anytime, so compatibility isn't guaranteed
 */
function createTableWithPlugins<Config, PluginState, Methods>(
  ...plugins: ReturnType<Plugin<Config, PluginState, Methods>>[]
) {
  return (config: BaseConfig<Row>) => {
    return batch(() => {
      const internalTable = createBaseTable<PluginState, Methods, Row>(config);
      plugins.reduce((table, plugin) => {
        plugin(table);
        return table;
      }, internalTable);

      // Don't expose internal API's like setState
      const { setState, internal, ...table } = internalTable;

      createComputed(() => {
        setState("rows", internalTable.internal.getVisibleRows());
      });
      return table;
    });
  };
}

function createBaseTable<S, M, R>(config: BaseConfig<R>) {
  config = klona(config);
  const [state, setState] = createStore<TableState<any>>({
    rows: [],
    _core: {
      config,
      data: [...config.data],
      visibleColumns: Object.keys(config.data[0]!).map((c) => ({
        key: c,
        header: c,
      })),
    },
    _plugins: {},
  });

  const ui = {
    Th: (props: JSX.HTMLElementTags["th"] & { header: Header }) => {
      const [_, rest] = splitProps(props, ["header", "children"]);
      return (
        <th
          {...rest}
          children={props.children || props.header.column.def.header}
        />
      );
    },
    Tr: (props: JSX.HTMLElementTags["tr"]) => <tr {...props} />,
    Td: (props: JSX.HTMLElementTags["td"] & { cell: Cell<any> }) => {
      const [_, rest] = splitProps(props, ["cell", "children"]);
      return <td {...rest}>{props.children || props.cell.getComponent()}</td>;
    },
  };

  const table = {
    state,
    setState,
    ui,
    internal: {
      getVisibleRows() {
        const data = state._core.data();
        return [...Array(data.length)].map((_, idx) => {
          return new Row(state, idx);
        });
      },
    },
  } as unknown as InternalTable<S, M>;
  return table;
}

export { createTableWithPlugins };
export { withPagination } from "./plugins/pagination";
export { withHeaders } from "./plugins/headers";
export { withResizableColumns } from "./plugins/resizableColumns";
export type { Plugin, Table };
