import { PgDialect } from "drizzle-orm/pg-core";
import { sql as drizzleSql } from "drizzle-orm";

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

export { sql };
export { and, eq, isNull, InferModel } from "drizzle-orm";
export {
  json,
  jsonb,
  pgTable,
  timestamp,
  varchar,
  boolean,
  text,
} from "drizzle-orm/pg-core";
export { drizzle } from "drizzle-orm/postgres-js";
