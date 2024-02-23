type ClientConfig =
  | {
      host: string;
      port: number;
      user: string;
      password: string;
      database: string;
      ssl?: boolean;
      // Query options

      /**
       * Whether to update column names to camel case
       */
      camelCase?: boolean;
    }
  | string;

type QueryResponse<Row> = {
  rowCount: number | null;
  rows: Row[];
  fields: { name: string; dataTypeID: number }[];
  modifiedRows?: number;
};

export type QueryOptions = {
  /**
   * Whether to rename column names to camel case
   * default: true
   */
  camelCase?: boolean;
};

type QueryClient = {
  query<Row>(
    sql: string,
    parameters?: any[],
    options?: QueryOptions
  ): Promise<QueryResponse<Row>>;
  query<Row>(query: {
    sql: string;
    params?: readonly any[];
    options?: QueryOptions;
  }): Promise<QueryResponse<Row>>;
};

export type Client = {
  connect(): Promise<void>;
  isConnected(): boolean;
  transaction<T>(closure: () => T | Promise<T>): Promise<void>;
  close(): void;
} & QueryClient;

type PoolOptions = ClientConfig & {
  max?: number;
  min?: number;
};

export type Pool = {
  connect(): Promise<Client & { release: () => Promise<void> }>;
} & QueryClient;

export const Pool: new (config: PoolOptions) => Pool;
export const Client: new (config: ClientConfig) => Client;
export type { PoolOptions, ClientConfig };
