// @ts-ignore
import { Client as PgClient } from "pg";
import { PgDialect } from "drizzle-orm/pg-core";
import { sql as drizzleSql } from "drizzle-orm";

/**
 * Only available in Arena cloud
 */
type QueryOptions = {
  /**
   * Whether to rename column names to camel case
   * default: true
   */
  camelCase?: boolean;
};

type ClientConfig = {
  credential: {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
  };

  /**
   * Whether to use the connection pool
   *
   * If not set, the connection will be initiated before executing the query
   * and termiated after the query is completed
   */
  pool?: number;
  options?: QueryOptions;
};

type Query = {
  /**
   * Raw query string
   */
  sql: string;
  /**
   * Query parameters
   */
  params: any[];
};

const executeQuery = async (config: ClientConfig & { query: Query }) => {
  const { query, credential } = config;
  const client = new PgClient({ credential });
  const result = await client.execute(query.sql, query.params);

  client.close();
  return result;
};

type SQLResult = { sql: string; params: any[] };
type SQL = (tag: TemplateStringsArray, ...args: any[]) => SQLResult;

const pg = new PgDialect();
const sql: SQL & typeof drizzleSql = Object.assign(
  (tag: TemplateStringsArray, ...args: any[]): SQLResult => {
    const sqlQuery = drizzleSql(tag, ...args);
    return pg.sqlToQuery(sqlQuery);
  },
  drizzleSql
);

export { sql, executeQuery };
