import { StoreSetter } from "@arena/solid-store";
import { DataSource } from "./data";

type withDefaultConfig<Source extends Record<"type" | "config", unknown>> = {
  type: Source["type"];
  default: Source["config"];
  config?: Source["config"];
};

export namespace Template {
  type dataConfigType<
    T extends Record<string, unknown>,
    Field extends keyof T
  > = Record<Field, DataFieldConfig<T[Field]>>;

  export type DataFieldConfig<T> = {
    title: string;
    description?: string;
    dataSource:
      | withDefaultConfig<DataSource.Transient<T>>
      | withDefaultConfig<DataSource.UserInput<T>>
      | withDefaultConfig<DataSource.Template<T>>
      | withDefaultConfig<DataSource.Dynamic<T>>;
  };

  export type DataConfig<T extends Record<string, unknown>> = dataConfigType<
    T,
    keyof T
  >;

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

  export type Props<Data extends Record<string, unknown>> = {
    /**
     * Widget id
     */
    id: string;
    attributes: any;
    data: Data;
    setData: StoreSetter<Data>;
  };
}
