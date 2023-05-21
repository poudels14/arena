import { ZodSchema, z } from "zod";

export const dataSourceTypeSchema = z.enum([
  /**
   * The data won't be stored anywhere in the backend but other widgets
   * can access the data. This data is usually provided by the template/widget
   * itself. For example, pagination state of a table, etc.
   */
  "transient",
  /**
   * Data comes from user input and won't be stored in the db but other
   * widgets can access the data
   */
  "userinput",
  /**
   * This data source is set by the template and can't be configured by the user.
   * This is used when storing data like Layout widget's children, etc
   */
  "template",
  /**
   * This data source is configured by the user.
   * TODO(sagar): better name for this?
   */
  "dynamic",
]);

export const dynamicDataSourceTypeSchema = z.enum([
  /**
   * Preview data provided by Widget Template
   */
  "preview",
  /**
   * Inline data provided by the user
   */
  "inline",
  /**
   * Data from parent widget
   */
  "parent",
  /**
   * Data returned by running a query in the serverss
   */
  "query",
]);

export const dataQueryTypesSchema = z.enum(["javascript", "sql"]);

export const dataTypeSchema = z.enum(["NUMBER", "TEXT", "LONGTEXT", "JSON"]);

export const previewSourceConfigSchema = z.object({
  source: z.literal("preview"),
  value: z.any(),
});

export const inlineSourceConfigSchema = z.object({
  source: z.literal("inline"),
  /**
   * Value should be any valid JS object that can be serialized
   */
  value: z.any(),
});

export const clientJsSourceConfigSchema = z.object({
  source: z.literal("client/js"),
  query: z.string(),
});

export const serverJsSourceConfigSchema = z.object({
  source: z.literal("server/js"),
  // TODO(sagar): save args as Object in db instead of {key, value} array
  /**
   * Query arguments
   */
  args: z.object({ key: z.string(), value: z.string() }).array(),
  query: z.string(),
});

export const serverSqlSourceConfigSchema = z.object({
  source: z.literal("server/sql"),
  /**
   * Database env var id
   */
  db: z.string(),
  // TODO(sagar): save args as Object in db instead of {key, value} array
  /**
   * Query arguments
   */
  args: z.object({ key: z.string(), value: z.string() }).array(),
  query: z.string(),
});

export const transientSourceSchema = z.object({
  type: z.literal("transient"),
  config: z.object({ value: z.any() }),
});

export const userInputSourceSchema = z.object({
  type: z.literal("userinput"),
  config: z.object({ value: z.any() }),
});

export const templateSourceSchema = z.object({
  type: z.literal("template"),
  config: z.union([
    clientJsSourceConfigSchema,
    serverJsSourceConfigSchema,
    serverSqlSourceConfigSchema,
  ]),
});

export const dynamicSourceSchema = z.object({
  type: z.literal("dynamic"),
  config: z.union([
    clientJsSourceConfigSchema,
    serverJsSourceConfigSchema,
    serverSqlSourceConfigSchema,
  ]),
});

export const dataSourceSchema = z.union([
  transientSourceSchema,
  userInputSourceSchema,
  templateSourceSchema,
  dynamicSourceSchema,
]) as ZodSchema<DataSource<any>>;

export type DataSourceType = z.infer<typeof dataSourceTypeSchema>;
export type DynamicDataSourceType = z.infer<typeof dynamicDataSourceTypeSchema>;
export type DataQueryTypes = z.infer<typeof dataQueryTypesSchema>;
export type DataType = z.infer<typeof dataTypeSchema>;

export namespace DataSource {
  type withValue<Shape, T> = Omit<Shape, "value"> & { value: T };

  export type Transient<T> = withValue<
    z.infer<typeof transientSourceSchema>,
    T
  >;

  export type UserInput<T> = withValue<
    z.infer<typeof userInputSourceSchema>,
    T
  >;

  export type PreviewSourceConfig<T> = withValue<
    z.infer<typeof previewSourceConfigSchema>,
    T
  >;
  export type InlineSourceConfig<T> = withValue<
    z.infer<typeof inlineSourceConfigSchema>,
    T
  >;
  export type ClientJsConfig = z.infer<typeof clientJsSourceConfigSchema>;
  export type ParentSourceConfig<T> = {
    /**
     * The field maps data from parent to the widget's data
     * For example, a widget takes in {id, name} as data but
     * parent passes {id, firstName}, then user can configure
     * the name field to map to "firstName" field from parent.
     * The config will be saved as { name: { field: "firstName "}}
     */
    field?: string;
  };

  export type ServerJsConfig = z.infer<typeof serverJsSourceConfigSchema>;
  export type ServerSqlConfig = z.infer<typeof serverSqlSourceConfigSchema>;
  export type Template<T> = {
    type: "template";
    config:
      | InlineSourceConfig<T>
      | ClientJsConfig
      | ServerJsConfig
      | ServerSqlConfig;
  };
  export type Dynamic<T> = {
    type: "dynamic";
    config:
      | InlineSourceConfig<T>
      | ClientJsConfig
      | ServerJsConfig
      | ServerSqlConfig;
  };
}

export type DataSource<T> =
  | DataSource.Transient<T>
  | DataSource.UserInput<T>
  | DataSource.Template<T>
  | DataSource.Dynamic<T>;
