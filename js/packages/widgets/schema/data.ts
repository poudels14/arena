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

export const contentTypeSchema = z
  .enum(["data", "asset/icon", "asset/image"])
  .default("data");

export const transientSourceSchema = z.object({
  source: z.literal("transient"),
  config: z.object({ value: z.any() }),
});

export const userInputSourceSchema = z.object({
  source: z.literal("userinput"),
  config: z.object({ value: z.any() }),
});

export const dataLoaderConfigSchema = z.union([
  z.object({
    loader: z.literal("@client/json"),
    value: z.any(),
  }),
  z.object({
    loader: z.literal("@client/js"),
    value: z.any(),
  }),
  z.object({
    loader: z.literal("@arena/sql/postgres"),
    /**
     * Id of the postgres database
     */
    db: z.string(),
    value: z.any(),

    /**
     * Metadata stored and used by the loader
     */
    metatada: z
      .object({
        /**
         * List of the names of JS objects accessed by the SQL query tempalte
         * For example, in `SELECT * FROM apps where id = {{ id }};`,
         * `args = ["id"]`
         */
        args: z.string().array(),
      })
      .optional(),
  }),
  z.object({
    loader: z.literal("@arena/server-function"),
    value: z.any(),

    /**
     * Metadata stored and used by the loader
     */
    metatada: z.any().optional(),
  }),
  // TODO(sp): TBD
  z.object({
    loader: z.literal("@arena/assets/icon"),
    /**
     * Id of the postgres database resource
     */
    resource: z.string(),
    /**
     * this is just to make TS happy
     */
    value: z.null(),
  }),
]);

export const templateSourceSchema = z.object({
  source: z.literal("template"),
  config: dataLoaderConfigSchema,
});

export const dynamicSourceSchema = z.object({
  source: z.literal("dynamic"),
  contentType: contentTypeSchema.optional(),
  config: dataLoaderConfigSchema,
});

export const dataSourceSchema = z.union([
  transientSourceSchema,
  userInputSourceSchema,
  templateSourceSchema,
  dynamicSourceSchema,
]) as ZodSchema<DataSource<any>>;

export type DataSource<T> =
  | DataSource.Transient<T>
  | DataSource.UserInput<T>
  | DataSource.Template
  | DataSource.Dynamic;

export namespace DataSource {
  type withValue<Shape, T> = Omit<Shape, "value"> & { value: T };

  export type Transient<T> = z.infer<typeof transientSourceSchema>;

  export type UserInput<T> = withValue<
    z.infer<typeof userInputSourceSchema>,
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

  export type Template = z.infer<typeof templateSourceSchema>;
  export type Dynamic = z.infer<typeof dynamicSourceSchema>;
}
