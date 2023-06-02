import {
  AnyColumn,
  SelectedFields,
  Table,
  sql as drizzleSql,
} from "drizzle-orm";
import { PgDialect } from "drizzle-orm/pg-core";
import { QueryBuilder } from "drizzle-orm/pg-core";

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

function select<T>(): ReturnType<QueryBuilder["select"]>;
function select<C extends AnyColumn, T extends Table>(
  fields?: SelectedFields<C, T>[]
) {
  const qb = new QueryBuilder();
  if (fields) {
    // @ts-expect-error
    return qb.select(fields);
  }
  return qb.select();
}

export { sql, select };
export { drizzle } from "./drizzle";
export * from "drizzle-orm/pg-core";
export {
  and,
  or,
  eq,
  isNotNull,
  isNull,
  like,
  notLike,
  ilike,
  notIlike,
  inArray,
  notInArray,
} from "drizzle-orm";
export type { InferModel } from "drizzle-orm";
