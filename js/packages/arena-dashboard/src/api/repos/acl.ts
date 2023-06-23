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
import { uniqueId } from "@arena/uikit/uniqueId";

export const acls = pgTable("acls", {
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
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type Acl = InferModel<typeof acls> & {
  access: AccessType;
  createdAt: Date;
  updatedAt?: Date | null;
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
        eq(acls.userId, filter.userId),
        isNull(acls.archivedAt),
      ];

      if (filter.workspaceId) {
        conditions.push(eq(acls.workspaceId, filter.workspaceId));
      }
      const rows = await db
        .select()
        .from(acls)
        .where(and(...conditions));
      return rows as Acl[];
    },
    async addAccess(
      acl: {
        workspaceId: string;
        userId: string;
        access: AccessType;
      } & (
        | { appId: string; path?: string }
        | {
            resourceId: string;
          }
      )
    ): Promise<Pick<Acl, "id" | "createdAt">> {
      const rows = await db
        .insert(acls)
        .values({
          id: uniqueId(),
          ...acl,
        })
        .returning({
          id: acls.id,
          createdAt: acls.createdAt,
        });
      return rows[0] as Pick<Acl, "id" | "createdAt">;
    },
  };
};

export { createRepo };
export type { Acl };
