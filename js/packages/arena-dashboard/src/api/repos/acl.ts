import {
  InferModel,
  and,
  drizzle,
  eq,
  isNull,
  pgTable,
  timestamp,
  varchar,
} from "@arena/db/pg";
import { Context } from "./context";
import { AccessType } from "../auth";

export const acl = pgTable("acl", {
  id: varchar("id").notNull(),
  /**
   * Special user ids:
   *
   * - "everyone": everyone in the workspace
   * - "public": shared publicly
   */
  userId: varchar("user_id").notNull(),
  workspaceId: varchar("workspace_id"),
  appId: varchar("app_id"),
  path: varchar("path"),
  resourceId: varchar("resource_id"),
  access: varchar("access").notNull(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type Acl = InferModel<typeof acl> & {
  access: AccessType;
  archivedAt?: Date | null;
};

const createRepo = (ctx: Context) => {
  const db = drizzle(ctx.client);
  return {
    async listAccess(filter: {
      userId: string;
      workspaceId: string | null | undefined;
    }): Promise<Required<Acl>[]> {
      const conditions = [
        eq(acl.userId, filter.userId),
        isNull(acl.archivedAt),
      ];

      if (filter.workspaceId) {
        conditions.push(eq(acl.workspaceId, filter.workspaceId));
      }
      const rows = await db
        .select()
        .from(acl)
        .where(and(...conditions));
      return rows as Acl[];
    },
  };
};

export { createRepo };
export type { Acl };
