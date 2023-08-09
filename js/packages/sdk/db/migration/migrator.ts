// @ts-expect-error
import { createHash } from "crypto";
import { SqliteDatabaseClient } from "../sqlite";
import { DbMigration } from "../common";
import { ArenaVectorDatabase, DatabaseClients, DatabaseConfig } from "..";

class DatabaseMigrator {
  #auditClient: SqliteDatabaseClient;
  #existingMigrations: DbMigration[];
  #dbName: DatabaseConfig["name"];
  #dbType: DatabaseConfig["type"];

  private constructor(
    auditClient: SqliteDatabaseClient,
    dbName: DatabaseConfig["name"],
    dbType: DatabaseConfig["type"],
    existingMigrations: DbMigration[]
  ) {
    this.#auditClient = auditClient;
    this.#dbName = dbName;
    this.#dbType = dbType;
    this.#existingMigrations = existingMigrations;
  }

  static async init(
    auditClient: SqliteDatabaseClient,
    dbName: DatabaseConfig["name"],
    dbType: DatabaseConfig["type"]
  ) {
    const { rows: existingMigrations } =
      await DatabaseMigrator.#getExistingMigrations(auditClient, {
        retry: 0,
        dbName,
        dbType,
      });

    return new DatabaseMigrator(
      auditClient,
      dbName,
      dbType,
      existingMigrations
    );
  }

  getMigrationClient(
    client: SqliteDatabaseClient | ArenaVectorDatabase.Client,
    migrationIndex: number
  ) {
    const self = this;
    return {
      async query<T>(sql: string, params?: any[]) {
        const hasher = createHash("sha256");
        const hash = hasher.update(sql).digest("hex");
        /**
         * Note(sagar): throw error if the migrations that already ran
         * were updated
         */
        if (migrationIndex < self.#existingMigrations.length) {
          let migrationAtIdx = self.#existingMigrations[migrationIndex];
          if (migrationAtIdx.hash != hash) {
            throw new Error(
              `Updating migration that already ran isn't allowed:\n` +
                `Changed query: ${sql}`
            );
          }
          if (migrationAtIdx.database != self.#dbName) {
            throw new Error(
              `Database name can't be changed.\n` +
                `Old: ${migrationAtIdx.database}, Updated: ${self.#dbName}`
            );
          }
          if (migrationAtIdx.type != self.#dbType) {
            throw new Error(
              `Database type can't be changed.\n` +
                `Old: ${migrationAtIdx.type}, Updated: ${self.#dbType}`
            );
          }
          console.log(
            `[setup] Skipping migration [db: ${
              self.#dbName
            }, index: ${migrationIndex}, reason: 'ran already']`
          );
          return;
        }

        console.log(
          `[setup] Running migration [db: ${self.#dbName}, type: ${
            self.#dbType
          }], index: ${migrationIndex}`
        );
        const res = await client.query<T>(sql, params);
        await self.#auditClient.query(
          `INSERT INTO _arena_schema_migrations (idx, database, type, hash) VALUES (?, ?, ?, ?)`,
          [migrationIndex, self.#dbName, self.#dbType, hash]
        );
        return res;
      },
    } as typeof client;
  }

  static async #createTable(auditClient: SqliteDatabaseClient) {
    auditClient.transaction(async () => {
      await auditClient.query(`
        CREATE TABLE _arena_schema_migrations (
          idx    INTEGER,
          database TEXT NOT NULL,
          type TEXT NOT NULL,
          hash  TEXT NOT NULL
        )`);

      await auditClient.query(`
        CREATE UNIQUE INDEX _arena_schema_migrations_unique_idx
          ON _arena_schema_migrations(idx, database, hash);
        )`);
    });
  }

  static async #getExistingMigrations(
    auditClient: SqliteDatabaseClient,
    options: {
      dbName: string;
      dbType: string;
      retry: number;
    }
  ): Promise<{ rows: DbMigration[] }> {
    return await auditClient
      .query<DbMigration>(
        `SELECT * FROM _arena_schema_migrations WHERE database = ? AND type = ?`,
        [options.dbName, options.dbType]
      )
      .catch(async (e: any) => {
        if (options.retry >= 1) {
          throw e;
        }
        if (e.message.includes("no such table")) {
          await DatabaseMigrator.#createTable(auditClient);
          return await DatabaseMigrator.#getExistingMigrations(auditClient, {
            ...options,
            retry: options.retry + 1,
          });
        }
        throw e;
      });
  }
}

export { DatabaseMigrator };
