type ClientConfig = {
  credential:
    | string
    | {
        host: string;
        port: string;
        username: string;
        password: string;
        database: string;
      };
};

type Client = {
  connect(): Promise<void>;
  isConnected(): boolean;

  query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
  query<T>(query: {
    sql: string;
    params: readonly any[];
  }): Promise<{ rows: T[] }>;
};

export const Client: new (config: ClientConfig) => Client;
