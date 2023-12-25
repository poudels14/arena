import { AbstractDatabaseConfig } from "../common";

export type PostgresDatabaseClient = {
  query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
  transaction<T>(closure: () => T | Promise<T>): Promise<void>;
};

export type PostgresDatabaseConfig = AbstractDatabaseConfig<{
  /**
   * Database type
   */
  type: "postgres";
}>;

export { PostgresDatabaseMigrator } from "./migrator";
