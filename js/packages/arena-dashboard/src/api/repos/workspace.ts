import {
  and,
  drizzle,
  eq,
  isNull,
  pgTable,
  timestamp,
  varchar,
} from "@arena/db/pg";
import { Context } from "./context";
import { WorkspaceAccessType } from "../auth/acl";

const workspaces = pgTable("workspaces", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
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
  access: WorkspaceAccessType;
};

const createRepo = (ctx: Context) => {
  const db = drizzle(ctx.client);
  return {
    async listWorkspaces(filter: {
      userId: string;
    }): Promise<Required<Workspace>[]> {
      const rows = await db
        .select({
          id: workspaces.id,
          name: workspaces.name,
          access: workspaceMembers.access,
        })
        .from(workspaces)
        .leftJoin(
          workspaceMembers,
          eq(workspaceMembers.workspaceId, workspaces.id)
        )
        .where(
          and(
            eq(workspaceMembers.userId, filter.userId),
            isNull(workspaceMembers.archivedAt),
            isNull(workspaces.archivedAt)
          )
        );
      return rows as Workspace[];
    },
  };
};

export { workspaceMembers, workspaces, createRepo };
export type { Workspace };
