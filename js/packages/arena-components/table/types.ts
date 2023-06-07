import { Store, StoreSetter } from "@arena/solid-store";
import { JSX, JSXElement } from "solid-js";
import { ColumnDef, Header } from "./column";
import { Cell, Row } from "./row";

type TableState<PluginsState> = {
  /** Interface to expose final rows */
  rows: Row[];

  _core: {
    config: BaseConfig<any>;

    /**
     * The copy of the passed in rows data that's used for rendering.
     * This data is the final result of filtering, sorting, etc.
     */
    data: any[];
    /**
     * All fields of the data is visible by default
     */
    visibleColumns: ColumnDef[];
  };

  /**
   * Internal state stored by plugins using plugin name as a key
   */
  _plugins: PluginsState;
};

type BaseConfig<Row> = {
  data: Row[];
};

type InternalTable<PluginsState, Methods> = {
  /**
   * Current state of the table
   */
  state: Store<TableState<PluginsState>>;

  setState: StoreSetter<TableState<PluginsState>>;

  setData: (data: any[]) => void;

  Ui: {
    Th: (props: JSX.HTMLElementTags["th"] & { header: Header }) => JSXElement;
    Tr: (props: JSX.HTMLElementTags["tr"]) => JSXElement;
    Td: (
      props: JSX.HTMLElementTags["td"] & { cell: Cell<unknown> }
    ) => JSXElement;
  };

  internal: {
    getVisibleRows: () => Row[];
  };
} & Methods;

type AnyInternalTable = InternalTable<unknown, unknown>;

type Table<S, M, R> = Pick<InternalTable<S, M>, "state" | "Ui" | "setData"> & M;

type Plugin<C, PS, M> = (config: C) => (table: InternalTable<PS, M>) => void;

export type {
  BaseConfig,
  InternalTable,
  TableState,
  AnyInternalTable,
  Table,
  Plugin,
};
