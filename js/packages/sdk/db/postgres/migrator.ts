import { SqlDatabaseClient } from "../sql";
import { AbstractDatabaseMigrator } from "../migration/migrator";
import { Migration } from "../common";

class PostgresDatabaseMigrator extends AbstractDatabaseMigrator {
  protected async createTable(auditClient: SqlDatabaseClient): Promise<void> {
    await auditClient.transaction(async () => {
      await auditClient.query(
        `
        CREATE TABLE IF NOT EXISTS _arena_schema_migrations (
          idx    INTEGER,
          database TEXT NOT NULL,
          type TEXT NOT NULL,
          hash  TEXT NOT NULL
        )`
      );

      await auditClient.query(`
        CREATE UNIQUE INDEX IF NOT EXISTS _arena_schema_migrations_unique_idx
          ON _arena_schema_migrations(idx, database, hash);
        `);
    });
  }

  protected async queryExistingMigrations(
    db: SqlDatabaseClient,
    options: {
      targetDbName: string;
      targetDbType: string;
    }
  ): Promise<{ rows: Migration[] }> {
    return await db.query<Migration>(
      `SELECT * FROM _arena_schema_migrations WHERE database = $1 AND type = $2`,
      [options.targetDbName, options.targetDbType]
    );
  }

  protected async insertMigration(
    db: SqlDatabaseClient,
    migration: Migration
  ): Promise<void> {
    await db.query(
      `INSERT INTO _arena_schema_migrations (idx, database, type, hash) VALUES ($1, $2, $3, $4)`,
      [migration.id, migration.database, migration.type, migration.hash]
    );
  }
}

export { PostgresDatabaseMigrator };
