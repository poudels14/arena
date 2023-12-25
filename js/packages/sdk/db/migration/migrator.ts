// @ts-ignore
import { createHash } from "crypto";
import { Migration } from "../common";
import { DatabaseClient, DatabaseConfig } from ".";
import { SqlDatabaseClient } from "../sql";

export type MigrationQueryRunner = {
  query<T>(sql: string, parameters?: any[]): Promise<T | null>;
};

abstract class AbstractDatabaseMigrator {
  #auditClient: SqlDatabaseClient;
  #targetDbName: DatabaseConfig["name"];
  #targetDbType: DatabaseConfig["type"];

  constructor(
    auditClient: SqlDatabaseClient,
    target: Pick<DatabaseConfig, "name" | "type">
  ) {
    this.#auditClient = auditClient;
    this.#targetDbName = target.name;
    this.#targetDbType = target.type;
  }

  async getMigrationClient(
    client: DatabaseClient,
    migrationIndex: number
  ): Promise<MigrationQueryRunner> {
    const self = this;
    const { rows: existingMigrations } = await self.#getExistingMigrations(
      self.#auditClient,
      {
        retry: 0,
        targetDbName: self.#targetDbName,
        targetDbType: self.#targetDbType,
      }
    );

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
          if (migrationAtIdx.database != self.#targetDbName) {
            throw new Error(
              `Database name can't be changed.\n` +
                `Old: ${migrationAtIdx.database}, Updated: ${
                  self.#targetDbName
                }`
            );
          }
          if (migrationAtIdx.type != self.#targetDbType) {
            throw new Error(
              `Database type can't be changed.\n` +
                `Old: ${migrationAtIdx.type}, Updated: ${self.#targetDbType}`
            );
          }
          console.log(
            `[setup] Skipping migration [db: ${
              self.#targetDbName
            }, index: ${migrationIndex}, reason: 'ran already']`
          );
          return null;
        }

        console.log(
          `[setup] Running migration [db: ${self.#targetDbName}, type: ${
            self.#targetDbName
          }], index: ${migrationIndex}`
        );
        const res = await client.query<T>(sql, params);
        await self.insertMigration(self.#auditClient, {
          id: migrationIndex,
          database: self.#targetDbName,
          type: self.#targetDbType,
          hash,
        });
        return res;
      },
    };
  }

  async #getExistingMigrations(
    auditClient: SqlDatabaseClient,
    options: {
      targetDbName: string;
      targetDbType: string;
      retry: number;
    }
  ): Promise<{ rows: Migration[] }> {
    return await this.queryExistingMigrations(auditClient, options).catch(
      async (e: any) => {
        if (options.retry >= 1) {
          throw e;
        }
        if (
          e.message.includes("no such table") ||
          // arena sql error
          e.message.includes("not found")
        ) {
          await this.createTable(auditClient).catch((e) => {
            console.error("Error creating migration table:", e);
            throw e;
          });
          return await this.#getExistingMigrations(auditClient, {
            ...options,
            retry: options.retry + 1,
          });
        }
        throw e;
      }
    );
  }

  protected abstract createTable(db: SqlDatabaseClient): Promise<void>;

  protected abstract queryExistingMigrations(
    db: SqlDatabaseClient,
    options: {
      targetDbName: string;
      targetDbType: string;
    }
  ): Promise<{ rows: Migration[] }>;

  protected abstract insertMigration(
    db: SqlDatabaseClient,
    migration: Migration
  ): Promise<void>;
}

export { AbstractDatabaseMigrator };
