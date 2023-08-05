import * as ArenaVectorDatabase from "../vectordb";
import { DatabaseMigrator } from "./migrator";
import { SqliteDatabaseClient } from "../sqlite";

class ArenaVectorDatabaseMigrator {
  #auditClient: SqliteDatabaseClient;
  constructor(auditClient: SqliteDatabaseClient) {
    this.#auditClient = auditClient;
  }

  async migrate(
    client: Pick<ArenaVectorDatabase.Client, "query">,
    config: ArenaVectorDatabase.Config
  ) {
    const migrator = await DatabaseMigrator.init(
      this.#auditClient,
      config.name,
      config.type
    );

    // TODO(sagar): add support for transactions in vector db?
    await Promise.all(
      config.migrations.map(async (m, idx) => {
        // @ts-expect-error
        await m.up(migrator.getMigrationClient(client, idx));
      })
    );
  }
}

export { ArenaVectorDatabaseMigrator };
