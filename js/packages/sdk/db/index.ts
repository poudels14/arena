import { SqliteDatabaseClient, SqliteDatabaseConfig } from "./sqlite";
import * as ArenaVectorDatabase from "./vectordb";
import { DatabaseMigrator } from "./migration/migrator";

type DatabaseConfig = SqliteDatabaseConfig | ArenaVectorDatabase.Config;

type DatabaseClients<Configs extends Record<string, DatabaseConfig>> = {
  [K in keyof Configs]: Configs[K]["type"] extends "sqlite"
    ? SqliteDatabaseClient
    : Configs[K]["type"] extends "arena-vectordb"
    ? ArenaVectorDatabase.Client
    : null;
};

const setupDatabase = async (
  auditClient: SqliteDatabaseClient,
  client: SqliteDatabaseClient | ArenaVectorDatabase.Client,
  config: DatabaseConfig
) => {
  const migrator = await DatabaseMigrator.init(
    auditClient,
    config.name,
    config.type
  );

  if (config.type == "sqlite") {
    (client as SqliteDatabaseClient).transaction(async () => {
      for (let idx = 0; idx < config.migrations.length; idx++) {
        const migration = config.migrations[idx];
        await migration.up(migrator.getMigrationClient(client, idx));
      }
    });
  } else if (config.type == "arena-vectordb") {
    // TODO(sagar): add support for transactions in vector db?
    for (let idx = 0; idx < config.migrations.length; idx++) {
      const migration = config.migrations[idx];
      // @ts-expect-error
      await migration.up(migrator.getMigrationClient(client, idx));
    }
  } else {
    throw new Error("Unsupported database type: " + (config as any).type);
  }
};

export { setupDatabase };
export type {
  SqliteDatabaseConfig,
  SqliteDatabaseClient,
  ArenaVectorDatabase,
  DatabaseConfig,
  DatabaseClients,
};
