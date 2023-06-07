import { StoreSetter } from "@arena/solid-store";
import { DataSource } from "./data";
import { z } from "zod";
import { JSX } from "solid-js";

export const templateSchema = z.object({
  /**
   * Id of the template
   */
  id: z.string(),

  /**
   * Name of the template
   */
  name: z.string(),

  /**
   * Url to load the component from
   */
  url: z.string(),
});

export namespace Template {
  type DataConfigWithValueType<
    T extends Record<string, unknown>,
    Field extends keyof T
  > = Record<Field, DataFieldConfig<T[Field]>>;

  type IncludingDefaults<T, S extends DataSource<T>> = Omit<S, "config"> & {
    default: S["config"];
    preview: T;
  };

  export type DataFieldConfig<T> = {
    title: string;
    description?: string;
  } & (
    | IncludingDefaults<T, DataSource.Transient<T>>
    | IncludingDefaults<T, DataSource.UserInput<T>>
    | IncludingDefaults<T, DataSource.Dynamic>
    | IncludingDefaults<T, DataSource.Template>
  );

  export type DataConfig<T extends Record<string, unknown>> =
    DataConfigWithValueType<T, keyof T>;

  export type Metadata<Data extends Record<string, unknown>> = {
    id: string;
    name: string;
    description: string;
    data: DataConfig<Data>;

    /**
     * Default tailwind classes for styling
     * These can be edited by the users
     */
    class?: string;

    /**
     * This is used by Widget templates to store non-data config.
     * For example, Table widget can use this field to store width of a column.
     */
    config?: any;
  };

  export type Props<Data extends Record<string, unknown>> = {
    /**
     * Widget id
     */
    id: string;
    attributes: any;
    data: Data;
    setData: StoreSetter<Data>;
    config?: any;
    setConfig: (config: any) => void;
    Editor: {
      Slot: (props: {
        parentId: string | null;

        /**
         * Whether the slot should contain a single widget of multiple widgets
         * Default: single
         */
        type?: "single" | "multiple";

        children?: JSX.Element;
      }) => JSX.Element;
    };
  };
}
