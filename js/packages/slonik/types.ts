import type * as tokens from "./tokens";

/**
 * @see https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-PARAMKEYWORDS
 */
export type ConnectionOptions = {
  applicationName?: string;
  databaseName?: string;
  host?: string;
  password?: string;
  port?: number;
  sslMode?: "disable" | "no-verify" | "require";
  username?: string;
};

export type TypeNameIdentifier =
  | "bool"
  | "bytea"
  | "float4"
  | "float8"
  | "int2"
  | "int4"
  | "int8"
  | "json"
  | "text"
  | "timestamptz"
  | "uuid";

export type SerializableValue =
  | boolean
  | number
  | string
  | readonly SerializableValue[]
  | {
      [key: string]: SerializableValue | undefined;
    }
  | null;

export type QueryId = string;

export type MaybePromise<T> = Promise<T> | T;

export type Connection = "EXPLICIT" | "IMPLICIT_QUERY" | "IMPLICIT_TRANSACTION";

export type Field = {
  readonly dataTypeId: number;
  readonly name: string;
};

export type QueryResult<T> = {
  readonly command: "COPY" | "DELETE" | "INSERT" | "SELECT" | "UPDATE";
  readonly fields: readonly Field[];
  // readonly notices: readonly Notice[],
  readonly rowCount: number;
  readonly rows: readonly T[];
};

export type QueryResultRowColumn = PrimitiveValueExpression;

export type QueryResultRow = Record<string, QueryResultRowColumn>;

export type Query = {
  readonly sql: string;
  readonly values: readonly PrimitiveValueExpression[];
};

export type SqlFragment = {
  readonly sql: string;
  readonly values: readonly PrimitiveValueExpression[];
};

/**
 * @property name Value of "pg_type"."typname" (e.g. "int8", "timestamp", "timestamptz").
 */
export type TypeParser<T = unknown> = {
  readonly name: string;
  readonly parse: (value: string) => T;
};

export type ArraySqlToken = {
  readonly memberType: SqlToken | TypeNameIdentifier | string;
  readonly type: typeof tokens.ArrayToken;
  readonly values: readonly PrimitiveValueExpression[];
};

export type BinarySqlToken = {
  readonly data: Buffer;
  readonly type: typeof tokens.BinaryToken;
};

export type IdentifierSqlToken = {
  readonly names: readonly string[];
  readonly type: typeof tokens.IdentifierToken;
};

export type ListSqlToken = {
  readonly glue: SqlSqlToken;
  readonly members: readonly ValueExpression[];
  readonly type: typeof tokens.ListToken;
};

export type JsonSqlToken = {
  readonly type: typeof tokens.JsonToken;
  readonly value: SerializableValue;
};

export type SqlSqlToken = {
  readonly sql: string;
  readonly type: typeof tokens.SqlToken;
  readonly values: readonly PrimitiveValueExpression[];
};

export type UnnestSqlColumn = string | readonly string[];

export type UnnestSqlToken = {
  readonly columnTypes: readonly UnnestSqlColumn[];
  readonly tuples: ReadonlyArray<readonly ValueExpression[]>;
  readonly type: typeof tokens.UnnestToken;
};

export type PrimitiveValueExpression =
  | Buffer
  | boolean
  | number
  | string
  | readonly PrimitiveValueExpression[]
  | null
  | any;

export type SqlToken =
  | ArraySqlToken
  | BinarySqlToken
  | IdentifierSqlToken
  | JsonSqlToken
  | ListSqlToken
  | SqlSqlToken
  | UnnestSqlToken;

export type ValueExpression = PrimitiveValueExpression | SqlToken;

export type NamedAssignment = {
  readonly [key: string]: ValueExpression;
};

// @todo may want to think how to make this extendable.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type UserQueryResultRow = Record<string, any>;

export type SqlTaggedTemplate<T extends UserQueryResultRow = QueryResultRow> = {
  <U extends UserQueryResultRow = T>(
    template: TemplateStringsArray,
    ...values: ValueExpression[]
  ): TaggedTemplateLiteralInvocation<U>;
  array: (
    values: readonly PrimitiveValueExpression[],
    memberType: SqlToken | TypeNameIdentifier
  ) => ArraySqlToken;
  binary: (data: Buffer) => BinarySqlToken;
  identifier: (names: readonly string[]) => IdentifierSqlToken;
  join: (
    members: readonly ValueExpression[],
    glue: SqlSqlToken
  ) => ListSqlToken;
  json: (value: SerializableValue) => JsonSqlToken;
  literalValue: (value: string) => SqlSqlToken;
  unnest: (
    // Value might be ReadonlyArray<ReadonlyArray<PrimitiveValueExpression>>,
    // or it can be infinitely nested array, e.g.
    // https://github.com/gajus/slonik/issues/44
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    tuples: ReadonlyArray<readonly any[]>,
    columnTypes: readonly UnnestSqlColumn[]
  ) => UnnestSqlToken;
};

// eslint-disable-next-line @typescript-eslint/no-empty-interface, @typescript-eslint/consistent-type-definitions, @typescript-eslint/no-unused-vars
export interface TaggedTemplateLiteralInvocation<
  Result extends UserQueryResultRow = QueryResultRow
> extends SqlSqlToken {}
