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

type ClientConfig = (
  | {
      host: string;
      port: number;
      database: string;
      user: string;
      password: string;
    }
  | {
      connectionString: string;
    }
) & {
  pool?: SqlDataQueryConfig["pool"];
  options?: QueryOptions;
};

class Client {
  client: typeof PgClient;
  constructor(config: ClientConfig) {
    this.client = new PgClient(config);
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

const executeQuery = async ({
  connectionString,
  query,
}: SqlDataQueryConfig) => {
  // TODO(sagar): end connection after query if pool isn't used
  return await connect({
    connectionString,
  }).execute(query.sql, query.params);
};

export { sql } from "@arena/db/pg";
export { connect, executeQuery };
export type { SqlDataQueryConfig };
