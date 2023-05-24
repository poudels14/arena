import { sql as drizzleSql } from "drizzle-orm";
import { PgDialect } from "drizzle-orm/pg-core";
import { QueryBuilder } from "drizzle-orm/pg-core";

type SQLResult = { sql: string; params: any[] };

const pg = new PgDialect();
const sql = (tag: TemplateStringsArray, ...args: any[]): SQLResult => {
  const sqlQuery = drizzleSql(tag, ...args);
  return pg.sqlToQuery(sqlQuery);
};

const select = () => {
  const qb = new QueryBuilder();
  // qb.select({
  //   name: drizzleSql`name`,
  // });
  throw new Error("not implemented");
};

export { sql, select };
