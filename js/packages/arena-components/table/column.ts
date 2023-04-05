import { JSXElement } from "solid-js";

type ColumnDef = {
  /**
   * Column accessor key
   */
  key: string;
  header: string | (() => JSXElement);
  cell?: (value: any) => JSXElement;
};

export { ColumnDef };
