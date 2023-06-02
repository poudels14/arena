import {
  createTableRelationsHelpers,
  extractTablesRelationalConfig,
  fillPlaceholders,
} from "drizzle-orm";
import {
  PgDatabase,
  PgDialect,
  PgSession,
  PreparedQuery,
  PreparedQueryConfig,
} from "drizzle-orm/pg-core";

/**
 * Note(sp): this was mostly copied from drizzle Neon integration
 */
class ArenaPreparedQuery<
  T extends PreparedQueryConfig
> extends PreparedQuery<T> {
  client: any;
  params: any;
  fields: any;
  customResultMapper: any;
  rawQuery: any;
  query: any;
  schema: any;
  options: any;
  constructor(
    client: any,
    queryString: string,
    params: any[],
    fields: any,
    name: string,
    customResultMapper: any
  ) {
    super();
    this.client = client;
    this.params = params;
    this.fields = fields;
    this.customResultMapper = customResultMapper;
    this.rawQuery = {
      name,
      text: queryString,
    };
    this.query = {
      name,
      text: queryString,
      rowMode: "array",
    };
  }
  async execute(placeholderValues = {}) {
    const params = fillPlaceholders(this.params, placeholderValues);
    const { fields, client, rawQuery, query, customResultMapper } = this;
    if (!fields && !customResultMapper) {
      return client.query(rawQuery.text, params);
    }

    const result = await client.query(query.text, params);
    return customResultMapper ? customResultMapper(result.rows) : result.rows;
  }
  all(placeholderValues = {}) {
    const params = fillPlaceholders(this.params, placeholderValues);
    return this.client
      .query(this.rawQuery, params)
      .then((result: any) => result.rows);
  }
  values(placeholderValues = {}) {
    const params = fillPlaceholders(this.params, placeholderValues);
    return this.client
      .query(this.query, params)
      .then((result: any) => result.rows);
  }
}

class ArenaPgSession extends PgSession {
  client: any;
  schema: any;
  options: any;
  constructor(client: any, dialect: any, schema: any, options = {}) {
    super(dialect);
    this.client = client;
    this.schema = schema;
    this.options = options;
  }
  prepareQuery<T>(
    query: any,
    fields: any,
    name: string,
    customResultMapper: any
  ) {
    return new ArenaPreparedQuery(
      this.client,
      query.sql,
      query.params,
      fields,
      name,
      customResultMapper
    );
  }
  // @ts-expect-error
  async transaction() {
    throw new Error("transaction not supported yet");
  }
}

function drizzle(client: any, config: { schema?: any } = {}) {
  const dialect = new PgDialect();
  let schema;
  if (config.schema) {
    const tablesConfig = extractTablesRelationalConfig(
      config.schema,
      createTableRelationsHelpers
    );
    schema = {
      fullSchema: config.schema,
      schema: tablesConfig.tables,
      tableNamesMap: tablesConfig.tableNamesMap,
    };
  }

  return new PgDatabase(
    dialect,
    // @ts-expect-error
    new ArenaPgSession(client, dialect, config.schema, {}),
    schema
  );
}

export { drizzle };
export * from "drizzle-orm/pg-core";
export { eq } from "drizzle-orm";
export type { InferModel } from "drizzle-orm";
