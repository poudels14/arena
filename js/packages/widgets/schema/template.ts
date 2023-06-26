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

  type IncludingDefaults<T, S extends DataSource<T>> = Omit<
    S,
    "config" | "value"
  > & {
    default: S["config"];
    preview: T;
  };

  export type DataFieldConfig<T> = {
    title: string;
    description?: string;
  } & (
    | Omit<DataSource.Transient<T>, "config" | "value">
    | IncludingDefaults<T, DataSource.UserInput<T>>
    | (Omit<DataSource.Config, "config" | "value"> & { default?: any })
    | IncludingDefaults<T, DataSource.Template>
    | IncludingDefaults<T, DataSource.Dynamic>
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
  };

  type DataSetter<Data> = Data extends Record<string, unknown>
    ? {
        [T in keyof Data as `set${Capitalize<string & T>}`]: (
          value: Data[T]
        ) => void;
      }
    : void;

  export type Editor = {
    useContext: () => {
      useResources: (filter?: {
        type?: string;
      }) => { id: string; name: string; type: string }[];
    };
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

  export type Props<Data extends Record<string, unknown>> = {
    /**
     * Widget id
     */
    id: string;
    attrs: any;
    Editor: Editor;
    isLoading(field: string): boolean;
  } & Data &
    DataSetter<Data>;
}
