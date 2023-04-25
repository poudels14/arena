export type DataSourceType =
  /**
   * The data won't be stored anywhere in the backend but other widgets
   * can access the data. This data is usually provided by the template/widget
   * itself. For example, pagination state of a table, etc.
   */
  | "transient"
  /**
   * Data comes from user input and won't be stored in the db but other
   * widgets can access the data
   */
  | "userinput"
  /**
   * This data source is set by the template and can't be configured by the user.
   * This is used when storing data like Layout widget's children, etc
   */
  | "template"
  /**
   * This data source is configured by the user.
   * TODO(sagar): better name for this?
   */
  | "dynamic";

export type DynamicDataSource =
  /**
   * Preview data provided by Widget Template
   */
  | "preview"
  /**
   * Inline data provided by the user
   */
  | "inline"
  /**
   * Data from parent widget
   */
  | "parent"
  /**
   * Data returned by running a query in the serverss
   */
  | "query";

export type DataQueryTypes = "javascript" | "sql";
export type DataType = "NUMBER" | "TEXT" | "LONGTEXT" | "JSON";

export namespace DataSources {
  export type sourceConfig<Type, Config> = Type extends Extract<
    DataSourceType,
    Type
  >
    ? {
        type: Type;
        config: Config;
      }
    : never;

  export type Transient<T> = sourceConfig<
    "transient",
    {
      value: T;
    }
  >;

  export type UserInput<T> = sourceConfig<
    "userinput",
    {
      value: T;
    }
  >;

  export type PreviewSourceConfig<T> = {
    source: "preview";
    /**
     * default: TEXT
     * TODO(sagar): is type necessary?
     */
    type?: DataType;
    value: T;
  };

  export type InlineSourceConfig<T> = {
    source: "inline";
    /**
     * default: TEXT
     * TODO(sagar): is type necessary?
     */
    type?: DataType;
    value: T;
  };

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

  export type JavascriptQuerySourceConfig = {
    source: "query";
    queryType: "javascript";
    // TODO(sagar): save args as Object in db instead of {key, value} array
    /**
     * Query arguments
     */
    args: { key: string; value: string }[];
    query: string;
  };

  export type SqlQuerySourceConfig = {
    source: "query";
    queryType: "sql";
    /**
     * Database env var id
     */
    db: string;
    // TODO(sagar): save args as Object in db instead of {key, value} array
    /**
     * Query arguments
     */
    args: { key: string; value: string }[];
    query: string;
  };

  export type Template<T> =
    | sourceConfig<"template", JavascriptQuerySourceConfig>
    | sourceConfig<"template", SqlQuerySourceConfig>
    | sourceConfig<"template", InlineSourceConfig<T>>;

  export type Dynamic<T> =
    | sourceConfig<"dynamic", PreviewSourceConfig<T>>
    | sourceConfig<"dynamic", InlineSourceConfig<T>>
    | sourceConfig<"dynamic", JavascriptQuerySourceConfig>
    | sourceConfig<"dynamic", SqlQuerySourceConfig>;
}

type iff<Type, SourceConfig> = Type extends Extract<DataSourceType, Type>
  ? SourceConfig
  : never;

export type DataSource<T> =
  | DataSources.Transient<T>
  | DataSources.UserInput<T>
  | DataSources.Template<T>
  | DataSources.Dynamic<T>;
