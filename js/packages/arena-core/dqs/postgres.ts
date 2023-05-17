// @ts-expect-error
import { Client as PgClient } from "pg";

type QueryOptions = {
  // Whether to rename column names to camel case
  // default: true
  camelCase?: boolean;
};

type ClientConfig = SecretConfig<{
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
}> & {
  options?: QueryOptions;
};

class Client {
  client: PgClient;
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
  return new Client(config);
};

export { connect };
