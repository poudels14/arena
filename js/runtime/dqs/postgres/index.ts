// @ts-ignore
import { Client as PgClient } from "pg";
import { PgDialect } from "drizzle-orm/pg-core";
import { sql as drizzleSql } from "drizzle-orm";
import { Pool, Client, ClientConfig } from "@arena/runtime/postgres";

declare var Arena;
const { core } = Arena;

class ArenasqlPool extends Pool {
  #connect: any;
  constructor(config) {
    super(config);
    this.#connect = super.connect;
  }

  // @ts-expect-error
  async connect() {
    throw new Error("Must use pool created with pool.withDefaultAclChecker()");
  }

  // @ts-expect-error
  async query() {
    throw new Error("Must use pool created with pool.withDefaultAclChecker()");
  }

  withDefaultAclChecker({ user }: { user: { id: string } }) {
    const self = this;
    return {
      async connect(): Promise<Client & { release(): Promise<void> }> {
        const client = await self.#connect();

        const clientRelease = client.release.bind(client);
        const clientQuery = client.query.bind(client);
        return Object.assign(client, {
          async query(query, params = undefined, options) {
            const finalQuery = core.ops.op_cloud_default_rowacl_apply_filters(
              user.id,
              query
            );
            return await clientQuery(finalQuery, params, options);
          },
          async release() {
            await clientRelease();
            Object.assign(client, {
              release: clientRelease,
              query: clientQuery,
            });
          },
        });
      },

      async query(...args: [any]) {
        const client = await this.connect();
        const res = await client.query(...args);
        await client.release();
        return res;
      },
    };
  }
}

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
  const { query } = config;
  const client = new PgClient(config);
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

export { ArenasqlPool as Pool, sql, executeQuery };
