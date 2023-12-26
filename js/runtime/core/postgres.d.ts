type ClientConfig = {
  credential:
    | string
    | {
        host: string;
        port: number;
        user: string;
        password: string;
        database: string;
        ssl?: boolean;
      };
};

type Client = {
  connect(): Promise<void>;
  isConnected(): boolean;

  query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
  query<T>(query: {
    sql: string;
    params?: readonly any[];
  }): Promise<{ rows: T[] }>;

  close(): void;
};

export const Client: new (config: ClientConfig) => Client;
