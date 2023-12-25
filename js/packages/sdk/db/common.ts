import { ArenaVectorDatabase } from ".";
import { MigrationQueryRunner } from "./migration/migrator";
import { PostgresDatabaseClient, PostgresDatabaseConfig } from "./postgres";
import { SqlDatabaseClient } from "./sql";
import { SqliteDatabaseClient, SqliteDatabaseConfig } from "./sqlite";

export type AbstractDatabaseConfig<Config> = {
  name: string;
  migrations: MigrationQuery[];
} & Config;

export type MigrationQuery = {
  up(db: MigrationQueryRunner): Promise<void>;
};

export type Migration = {
  id: number;
  /**
   * Name of the database
   */
  database: string;
  /**
   * Database type
   */
  type: string;
  hash: string;
};

export type DatabaseConfig =
  | PostgresDatabaseConfig
  | SqliteDatabaseConfig
  | ArenaVectorDatabase.Config;

export type DatabaseClient = SqlDatabaseClient | ArenaVectorDatabase.Client;

export type DatabaseClients<Configs extends Record<string, DatabaseConfig>> = {
  [K in keyof Configs]: Configs[K]["type"] extends "sqlite"
    ? SqliteDatabaseClient
    : Configs[K]["type"] extends "postgres"
    ? PostgresDatabaseClient
    : Configs[K]["type"] extends "arena-vectordb"
    ? ArenaVectorDatabase.Client
    : null;
};
