import { AbstractDatabaseConfig } from "../common";

export type SqliteDatabaseClient = {
  query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
  transaction<T>(closure: () => T | Promise<T>): Promise<void>;
};

export type SqliteDatabaseConfig = AbstractDatabaseConfig<{
  /**
   * Database type
   */
  type: "sqlite";
}>;
