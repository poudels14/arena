import { AbstractDatabaseMigrator } from "../migration/migrator";
import { DatabaseClient, DatabaseConfig } from "..";

const runDatabaseMigration = async (
  migrator: AbstractDatabaseMigrator,
  client: DatabaseClient,
  config: DatabaseConfig
) => {
  let supportedDb =
    config.type == "sqlite" ||
    config.type == "postgres" ||
    config.type == "arena-vectordb";

  if (!supportedDb) {
    throw new Error("Unsupported database type: " + (config as any).type);
  }

  // Note(sagar): run all new migrations in a single transaction so that
  // when upgrading app version, the database doesn't get stuck in the middle
  // of versions. So, we need to make sure either all new migrations run
  // successfully so that the app can be upgraded to the new version, or we
  // rollback and continue using old version
  await client.transaction(async () => {
    for (let idx = 0; idx < config.migrations.length; idx++) {
      const migration = config.migrations[idx];
      await migration.up(await migrator.getMigrationClient(client, idx));
    }
  });
};

export { runDatabaseMigration };
export type { DatabaseConfig, DatabaseClient };
