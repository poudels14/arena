import { JSXElement } from "solid-js";

type Header = {
  id: string;
  colSpan: number;
  column: {
    def: ColumnDef;
  };
};

type ColumnDef = {
  /**
   * Column accessor key
   */
  key: string;
  header: string | JSXElement;
  cell?: (value: any) => JSXElement;
};

export type { Header, ColumnDef };
