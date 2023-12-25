type QueryClient = {
  query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
};

export type SqlDatabaseClient = {
  transaction<T>(closure: () => T | Promise<T>): Promise<void>;
} & QueryClient;
