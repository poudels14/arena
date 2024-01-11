import { and, eq } from "drizzle-orm";
import { json, pgTable, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

const databases = pgTable("databases", {
  id: varchar("id").notNull(),
  workspace_id: varchar("workspace_id").notNull(),
  app_id: varchar("app_id"),
  credentials: json("credentials"),
  cluster_id: varchar("cluster_id"),
});

type Database = {
  id: string;
  workspaceId: string;
  appId: string | null;
  // credentials of the app user
  credentials: {
    user: string;
    password: string;
  };
  clusterId: string | null;
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async getById(id: string): Promise<Database | null> {
      const rows = await db
        .with()
        .select({
          id: databases.id,
          workspaceId: databases.workspace_id,
          appId: databases.app_id,
          credentials: databases.credentials,
          clusterId: databases.cluster_id,
        })
        .from(databases)
        .where(and(eq(databases.id, id)));

      return (rows[0] as Database) || null;
    },
    async list(filters: { workspaceId: string }): Promise<Database[]> {
      const rows = await db
        .select({
          id: databases.id,
          workspaceId: databases.workspace_id,
          appId: databases.app_id,
          credentials: databases.credentials,
          clusterId: databases.cluster_id,
        })
        .from(databases)
        .where(and(eq(databases.workspace_id, filters.workspaceId)));
      return rows as Database[];
    },
    async add(options: {
      id: string;
      workspaceId: string;
      appId: string | null;
      clusterId: string | null;
      credentials: Database["credentials"];
    }): Promise<Required<Database>> {
      await db.insert(databases).values({
        id: options.id,
        workspace_id: options.workspaceId,
        app_id: options.appId || null,
        cluster_id: options.clusterId || null,
        credentials: options.credentials,
      });

      const rows = await db
        .select({
          id: databases.id,
          workspaceId: databases.workspace_id,
          appId: databases.app_id || null,
          credentials: databases.credentials,
          clusterId: databases.cluster_id || null,
        })
        .from(databases)
        .where(and(eq(databases.id, options.id)));
      return rows[0] as any as Database;
    },
  };
};

export { createRepo };
export type { Database };
