import { and, eq, isNull } from "drizzle-orm";
import { json, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { WorkspaceAccessType } from "@portal/sdk/acl";

const workspaces = pgTable("workspaces", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
  config: json("config"),
  createdAt: timestamp("created_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

const workspaceMembers = pgTable("workspace_members", {
  workspaceId: varchar("workspace_id").notNull(),
  userId: varchar("user_id").notNull(),
  access: varchar("access").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type Workspace = {
  id: string;
  name: string;
  config: {
    // runtime: any;
    // databaseHost: string;
  };
  access: WorkspaceAccessType;
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async getWorkspaceById(id: string): Promise<Required<Workspace> | null> {
      const rows = await db
        .with()
        .select({
          id: workspaces.id,
          name: workspaces.name,
          config: workspaces.config,
          access: workspaceMembers.access,
        })
        .from(workspaces)
        .leftJoin(
          workspaceMembers,
          eq(workspaceMembers.workspaceId, workspaces.id)
        )
        .where(
          and(
            eq(workspaces.id, id),
            isNull(workspaceMembers.archivedAt),
            isNull(workspaces.archivedAt)
          )
        );

      return (rows[0] as Workspace) || null;
    },
    async listWorkspaces(
      filter: {
        userId?: string;
      },
      pagination: { limit?: number; offset?: number } = {}
    ): Promise<Required<Workspace>[]> {
      const rows = await db
        .select({
          id: workspaces.id,
          name: workspaces.name,
          config: workspaces.config,
          access: workspaceMembers.access,
          createdAt: workspaces.createdAt,
        })
        .from(workspaces)
        .leftJoin(
          workspaceMembers,
          eq(workspaceMembers.workspaceId, workspaces.id)
        )
        .where(
          filter.userId
            ? and(
                eq(workspaceMembers.userId, filter.userId),
                isNull(workspaceMembers.archivedAt),
                isNull(workspaces.archivedAt)
              )
            : and(isNull(workspaces.archivedAt))
        )
        .limit(pagination.limit || 500)
        .offset(pagination.offset || 0);
      return rows as Workspace[];
    },
    async createWorkspace(options: {
      id: string;
      name?: string;
      ownerId: string;
    }): Promise<Required<Workspace>> {
      const workspace = {
        id: options.id,
        name: options.name || "Default workspace",
        config: {},
        createdAt: new Date(),
      };

      await db.insert(workspaces).values(workspace).execute();
      await db.insert(workspaceMembers).values({
        workspaceId: options.id,
        userId: options.ownerId,
        access: "owner",
      });

      return {
        ...workspace,
        access: "owner",
      };
    },
  };
};

export { workspaceMembers, workspaces, createRepo };
export type { Workspace };
