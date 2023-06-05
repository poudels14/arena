import { Client as PgClient } from "pg";
import { SqlDataQueryConfig } from ".";

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
  pool?: SqlDataQueryConfig["pool"];
  options?: QueryOptions;
};

class Client {
  client: typeof PgClient;
  constructor(config: ClientConfig) {
    const { credential } = config;
    this.client = new PgClient({
      ...config,
      ...(credential.host ? credential : {}),
    });
  }

  async execute(query: string, parameters: any[], options?: QueryOptions) {
    // Note(sagar): this is applicable to @arena cloud pg module
    if (this.client.isConnected && !this.client.isConnected()) {
      await this.client.connect();
    } else {
      // needed for npm `pg` module
      await this.client.connect();
    }
    return await this.client.query(query, parameters, options);
  }
}

const connect = (config: ClientConfig) => {
  // TODO(sp): create connection pool
  return new Client(config);
};

const executeQuery = async (config: SqlDataQueryConfig) => {
  // TODO(sagar): end connection after query if pool isn't used
  const { query, credential } = config;
  return await connect({ credential }).execute(query.sql, query.params);
};

export { connect, executeQuery };
export { sql } from "@arena/db/pg";
export type { SqlDataQueryConfig };
