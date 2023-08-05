import { SqliteDatabaseClient, SqliteDatabaseConfig } from "../sqlite";
import { DatabaseMigrator } from "./migrator";

class SqliteMigrator {
  #auditClient: SqliteDatabaseClient;
  constructor(auditClient: SqliteDatabaseClient) {
    this.#auditClient = auditClient;
  }

  async migrate(client: SqliteDatabaseClient, config: SqliteDatabaseConfig) {
    const migrator = await DatabaseMigrator.init(
      this.#auditClient,
      config.name,
      config.type
    );

    client.transaction(async () => {
      await Promise.all(
        config.migrations.map(async (m, idx) => {
          await m.up(migrator.getMigrationClient(client, idx));
        })
      );
    });
  }
}

export { SqliteMigrator };
