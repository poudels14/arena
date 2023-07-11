import type { DatabaseClient, DbMigration, MigrationQuery } from "../../db";
import { createHash } from "crypto";

class SqliteMigrator {
  #client: DatabaseClient;
  constructor(client: DatabaseClient) {
    this.#client = client;
  }

  async migrate(migrationQueries: MigrationQuery[]) {
    const { rows: existingMigrations } = await this.#getExistingMigrations({
      retry: 0,
    });

    const self = this;
    const createMigrationClient = (migrationIndex: number) => {
      return {
        async query<T>(sql: string, params?: any[]) {
          const hasher = createHash("sha256");
          const hash = hasher.update(sql).digest("hex");
          /**
           * Note(sagar): throw error if the migrations that already ran
           * were updated
           */
          if (migrationIndex < existingMigrations.length) {
            if (existingMigrations[migrationIndex].hash != hash) {
              throw new Error(
                `Updating migration that already ran isn't allowed:\n` +
                  `Changed query: ${sql}`
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
            `INSERT INTO _arena_schema_migrations (id, hash) VALUES (?, ?)`,
            [migrationIndex, hash]
          );
          return res;
        },
      } as Pick<DatabaseClient, "query">;
    };

    this.#client.transaction(async () => {
      await Promise.all(
        migrationQueries.map(async (m, idx) => {
          m.up(createMigrationClient(idx));
        })
      );
    });
  }

  async #createTable() {
    this.#client.transaction(async () => {
      await this.#client.query(`
        CREATE TABLE _arena_schema_migrations (
          id    INTEGER PRIMARY KEY,
          hash  TEXT NOT NULL
        )`);

      await this.#client.query(`
        CREATE UNIQUE INDEX _arena_schema_migrations_unique_idx
          ON _arena_schema_migrations(id, hash);
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
