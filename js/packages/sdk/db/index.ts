import { SqlDatabaseClient, SqliteDatabaseConfig } from "./sqlite";
import * as ArenaVectorDatabase from "./vectordb";
import { SqliteMigrator } from "./migration/sqlite";

type DatabaseConfig = SqliteDatabaseConfig | ArenaVectorDatabase.Config;

type DatabaseClients<Configs extends Record<string, DatabaseConfig>> = {
  [K in keyof Configs]: Configs[K]["type"] extends "sqlite"
    ? SqlDatabaseClient
    : Configs[K]["type"] extends "arena-vectordb"
    ? ArenaVectorDatabase.Client
    : null;
};

const setupSqliteDatabase = async (options: {
  client: SqlDatabaseClient;
  config: SqliteDatabaseConfig;
}) => {
  console.log("[setup] Running sqlite database migrations...");
  await new SqliteMigrator(options.client).migrate(options.config);
};

const setupArenaVectorDatabase = async (options: {
  client: ArenaVectorDatabase.Client;
  config: ArenaVectorDatabase.Config;
}) => {
  throw new Error("Arena vector database migration not yet supported");
};

export { setupSqliteDatabase, setupArenaVectorDatabase, SqliteMigrator };
export type {
  SqliteDatabaseConfig,
  SqlDatabaseClient,
  ArenaVectorDatabase,
  DatabaseConfig,
  DatabaseClients,
};
