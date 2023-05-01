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
  /**
   * default: TEXT
   * TODO(sagar): is type necessary?
   */
  type: dataTypeSchema.optional(),
  value: z.any(),
});

export const inlineSourceConfigSchema = z.object({
  source: z.literal("inline"),
  /**
   * default: TEXT
   * TODO(sagar): is type necessary?
   */
  type: dataTypeSchema.optional(),
  value: z.any(),
});

export const javascriptQuerySourceConfigSchema = z.object({
  source: z.literal("query"),
  queryType: z.literal("javascript"),
  // TODO(sagar): save args as Object in db instead of {key, value} array
  /**
   * Query arguments
   */
  args: z.object({ key: z.string(), value: z.string() }).array(),
  query: z.string(),
});

export const sqlQuerySourceConfigSchema = z.object({
  source: z.literal("query"),
  queryType: z.literal("sql"),
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
    inlineSourceConfigSchema,
    javascriptQuerySourceConfigSchema,
    sqlQuerySourceConfigSchema,
  ]),
});

export const dynamicSourceSchema = z.object({
  type: z.literal("dynamic"),
  config: z.union([
    inlineSourceConfigSchema,
    javascriptQuerySourceConfigSchema,
    sqlQuerySourceConfigSchema,
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

export namespace DataSources {
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

  export type JavascriptQuerySourceConfig = z.infer<
    typeof javascriptQuerySourceConfigSchema
  >;
  export type SqlQuerySourceConfig = z.infer<typeof sqlQuerySourceConfigSchema>;
  export type Template<T> = {
    type: "template";
    config:
      | InlineSourceConfig<T>
      | JavascriptQuerySourceConfig
      | SqlQuerySourceConfig;
  };
  export type Dynamic<T> = {
    type: "dynamic";
    config:
      | InlineSourceConfig<T>
      | JavascriptQuerySourceConfig
      | SqlQuerySourceConfig;
  };
}

export type DataSource<T> =
  | DataSources.Transient<T>
  | DataSources.UserInput<T>
  | DataSources.Template<T>
  | DataSources.Dynamic<T>;
