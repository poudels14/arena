import { createHash } from "crypto";
import { SqlDatabaseClient, SqliteDatabaseConfig } from "../sqlite";
import { DbMigration } from "../common";

class SqliteMigrator {
  #client: SqlDatabaseClient;
  constructor(client: SqlDatabaseClient) {
    this.#client = client;
  }

  async migrate(config: SqliteDatabaseConfig) {
    const { rows: existingMigrations } = await this.#getExistingMigrations({
      retry: 0,
    });

    const self = this;
    const createMigrationClient = (
      dbName: SqliteDatabaseConfig["name"],
      dbType: SqliteDatabaseConfig["type"],
      migrationIndex: number
    ) => {
      return {
        async query<T>(sql: string, params?: any[]) {
          const hasher = createHash("sha256");
          const hash = hasher.update(sql).digest("hex");
          /**
           * Note(sagar): throw error if the migrations that already ran
           * were updated
           */
          if (migrationIndex < existingMigrations.length) {
            let migrationAtIdx = existingMigrations[migrationIndex];
            if (migrationAtIdx.hash != hash) {
              throw new Error(
                `Updating migration that already ran isn't allowed:\n` +
                  `Changed query: ${sql}`
              );
            }
            if (migrationAtIdx.database != dbName) {
              throw new Error(
                `Database name can't be changed.\n` +
                  `Old: ${migrationAtIdx.database}, Updated: ${dbName}`
              );
            }
            if (migrationAtIdx.type != dbType) {
              throw new Error(
                `Database type can't be changed.\n` +
                  `Old: ${migrationAtIdx.type}, Updated: ${dbType}`
              );
            }
            console.log(
              `[setup] Skipping migration ${migrationIndex} [reason='ran already']`
            );
            return;
          }

          console.log("[setup] Running migration:", migrationIndex);
          const res = await self.#client.query<T>(sql, params);
          await self.#client.query(
            `INSERT INTO _arena_schema_migrations (id, database, type, hash) VALUES (?, ?, ?, ?)`,
            [migrationIndex, dbName, dbType, hash]
          );
          return res;
        },
      } as Pick<SqlDatabaseClient, "query">;
    };

    this.#client.transaction(async () => {
      await Promise.all(
        config.migrations.map(async (m, idx) => {
          await m.up(createMigrationClient(config.name, config.type, idx));
        })
      );
    });
  }

  async #createTable() {
    this.#client.transaction(async () => {
      await this.#client.query(`
        CREATE TABLE _arena_schema_migrations (
          id    INTEGER PRIMARY KEY,
          database TEXT NOT NULL,
          type TEXT NOT NULL,
          hash  TEXT NOT NULL
        )`);

      await this.#client.query(`
        CREATE UNIQUE INDEX _arena_schema_migrations_unique_idx
          ON _arena_schema_migrations(id, database, hash);
        )`);
    });
  }

  async #getExistingMigrations(options: {
    retry: number;
  }): Promise<{ rows: DbMigration[] }> {
    return await this.#client
      .query<DbMigration>(`SELECT * FROM _arena_schema_migrations`)
      .catch(async (e: any) => {
        if (options.retry >= 1) {
          throw e;
        }
        if (e.message.includes("no such table")) {
          await this.#createTable();
          return await this.#getExistingMigrations({
            retry: options.retry + 1,
          });
        }
        throw e;
      });
  }
}

export { SqliteMigrator };
