import { $RAW, Store } from "@arena/solid-store";
import { Match, Switch, children } from "solid-js";
import CheckIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/small-tick";
import { ColumnDef } from "./column";
import { TableState } from "./types";
import { InlineIcon } from "../InlineIcon";

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
    const value = this.getValue();
    return this.columnDef.cell ? (
      children(() => this.columnDef.cell!(value))
    ) : (
      <Switch>
        <Match when={typeof value == "boolean"}>
          <div class="flex justify-center">
            <InlineIcon size="18px">
              <path d={CheckIcon[0]} />
            </InlineIcon>
          </div>
        </Match>
        <Match when={true}>{value as string}</Match>
      </Switch>
    );
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
