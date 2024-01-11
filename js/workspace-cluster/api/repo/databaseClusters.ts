import { and, eq } from "drizzle-orm";
import { integer, json, pgTable, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

const databaseClusters = pgTable("database_clusters", {
  id: varchar("id").notNull(),
  host: varchar("host").notNull(),
  port: integer("port").notNull(),
  capacity: integer("capacity").notNull(),
  usage: integer("usage").notNull(),
  credentials: json("credentials"),
});

type DatabaseCluster = {
  id: string;
  host: string;
  port: number;
  capacity: number;
  usage: number;
  // credentials of the admin user
  credentials: {
    adminUser: string;
    adminPassword: string;
  };
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async getById(id: string): Promise<Required<DatabaseCluster> | null> {
      const rows = await db
        .with()
        .select({
          id: databaseClusters.id,
          credentials: databaseClusters.credentials,
        })
        .from(databaseClusters)
        .where(and(eq(databaseClusters.id, id)));

      return (rows[0] as DatabaseCluster) || null;
    },
    async list(): Promise<Required<DatabaseCluster>[]> {
      const rows = await db
        .select({
          id: databaseClusters.id,
          host: databaseClusters.host,
          port: databaseClusters.port,
          capacity: databaseClusters.capacity,
          usage: databaseClusters.usage,
          credentials: databaseClusters.credentials,
        })
        .from(databaseClusters);
      return rows as DatabaseCluster[];
    },
    async add(options: {
      id: string;
      host: string;
      port: number;
      capacity: number;
      credentials: DatabaseCluster["credentials"];
    }): Promise<Required<DatabaseCluster>> {
      await db.insert(databaseClusters).values({
        id: options.id,
        capacity: options.capacity,
        host: options.host,
        port: options.port,
        usage: 0,
        credentials: options.credentials,
      });

      const cluster = await this.getById(options.id);
      return cluster!;
    },
    async delete(options: { id: string }): Promise<void> {
      await db
        .delete(databaseClusters)
        .where(and(eq(databaseClusters.id, options.id)));
    },
  };
};

export { createRepo };
export type { DatabaseCluster };
