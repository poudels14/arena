import { Store } from "@arena/solid-store";
import { children } from "solid-js";
import { ColumnDef } from "./column";
import { TableState } from "./types";

type State = Store<TableState<any>>;

class Cell<T> {
  data: any[];
  rowIndex: number;
  columnDef: ColumnDef;
  constructor(data: any[], rowIndex: number, columnDef: ColumnDef) {
    this.data = data;
    this.rowIndex = rowIndex;
    this.columnDef = columnDef;
  }

  getValue(): T {
    const row = this.data[this.rowIndex];
    return row[this.columnDef.key];
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
  constructor(state: State, index: number) {
    this.state = state;
    this.index = index;
  }

  getVisibleCells() {
    const visibleColumns = this.state._core.visibleColumns();
    const data = this.state._core.data();
    return visibleColumns.map((column) => {
      return new Cell<any>(data, this.index, column);
    });
  }
}

export { Cell, Row };
