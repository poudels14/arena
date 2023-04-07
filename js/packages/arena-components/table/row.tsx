import { $RAW, Store } from "@arena/solid-store";
import { children } from "solid-js";
import { ColumnDef } from "./column";
import { TableState } from "./types";

type State = Store<TableState<any>>;

class Cell<T> {
  row: Row;
  columnDef: ColumnDef;
  constructor(row: Row, columnDef: ColumnDef) {
    this.row = row;
    this.columnDef = columnDef;
  }

  getValue(): T {
    return this.row.value[this.columnDef.key];
  }

  getComponent() {
    return this.columnDef.cell
      ? children(() => this.columnDef.cell!(this.getValue()))
      : this.getValue();
  }
}

class Row {
  state: State;
  index: number;
  value: any;
  constructor(state: State, index: number) {
    this.state = state;
    this.index = index;
    this.value = state._core.data[$RAW][index];
  }

  getVisibleCells() {
    const visibleColumns = this.state._core.visibleColumns();
    const row = this;
    return visibleColumns.map((column) => {
      return new Cell<any>(row, column);
    });
  }
}

export { Cell, Row };
