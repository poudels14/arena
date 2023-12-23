export const Flags: {
  SQLITE_OPEN_READ_ONLY: 1;
  SQLITE_OPEN_READ_WRITE: 2;
  SQLITE_OPEN_CREATE: 4;
  SQLITE_OPEN_URI: 64;
  SQLITE_OPEN_NO_MUTEX: 32768;
  SQLITE_OPEN_NOFOLLOW: 0x0100_0000;
};

type ClientConfig = {
  path: String;
  flags?: number;
  options?: {
    camelCase?: boolean;
  };
};

type Client = {
  query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
  query<T>(query: {
    sql: string;
    params: readonly any[];
  }): Promise<{ rows: T[] }>;
  transaction<T>(closure: () => T | Promise<T>): Promise<void>;
  close(): Promise<void>;
};

export const Client: new (config: ClientConfig) => Client;
