import { Store, StoreSetter } from "@arena/solid-store";
import { ColumnDef } from "./column";
import { Row } from "./row";

type TableState<PluginsState> = {
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

type InternalTable<PluginsState, Methods, R> = {
  /**
   * Current state of the table
   */
  state: Store<TableState<PluginsState>>;

  setState: StoreSetter<TableState<PluginsState>>;

  internal: {
    getVisibleRows: () => Row[];
  };
} & Methods;

type AnyInternalTable = InternalTable<unknown, unknown, unknown>;

type Table<S, M, R> = Pick<InternalTable<S, M, R>, "state"> & M;

type Plugin<C, PS, M> = (
  config: C
) => (table: InternalTable<PS, M, any>) => void;

export type {
  BaseConfig,
  InternalTable,
  TableState,
  AnyInternalTable,
  Table,
  Plugin,
};
