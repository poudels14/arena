import { StoreSetter } from "@arena/solid-store";
import { DataSources } from "./data";

type withDefaultConfig<Source extends Record<"type" | "config", unknown>> = {
  type: Source["type"];
  default: Source["config"];
  config?: Source["config"];
};

export namespace Template {
  export type DataFieldConfig<T> = {
    title: string;
    description?: string;
    dataSource:
      | withDefaultConfig<DataSources.Transient<T>>
      | withDefaultConfig<DataSources.UserInput<T>>
      | withDefaultConfig<DataSources.Template<T>>
      | withDefaultConfig<DataSources.Dynamic<T>>;
  };

  type dataConfigType<
    T extends Record<string, unknown>,
    Field extends keyof T
  > = Record<Field, DataFieldConfig<T[Field]>>;

  export type DataConfig<T extends Record<string, unknown>> = dataConfigType<
    T,
    keyof T
  >;
}

export type TemplateMetadata<Data extends Record<string, unknown>> = {
  id: string;
  name: string;
  description: string;
  data: Template.DataConfig<Data>;
};

export type TemplateProps<Data extends Record<string, unknown>> = {
  attributes: any;
  data: Data;
  setData: StoreSetter<Data>;
};
