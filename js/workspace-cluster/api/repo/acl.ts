import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { InferModel, and, eq, isNull } from "drizzle-orm";
import { pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

export const acls = pgTable("acls", {
  id: varchar("id").notNull(),
  workspaceId: varchar("workspace_id"),
  /**
   * Special user ids:
   *
   * - "everyone": everyone in the workspace
   * - "public": shared publicly
   */
  userId: varchar("user_id").notNull(),
  access: varchar("access").notNull(),
  appId: varchar("app_id"),
  path: varchar("path"),
  resourceId: varchar("resource_id"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type AccessType =
  /**
   * This access allows user to view an app (i.e. run GET queries),
   * view resources, use resources in an app for fetching data but prevent from
   * running mutate action on app or using resources for mutate action;
   */
  | "view-entity"
  /**
   * This access allows user to run mutate actions of the app, or run mutate
   * actions on resources (for eg, INSERT/UPDATE on postgres db).
   *
   * If a user has access on an app but not on a resource, only the queries
   * in the app can be run by the user and can't use the resource directly
   */
  | "mutate-entity"
  /**
   * This access allows user to edit an app, a resource, etc
   */
  | "admin"
  /**
   * The owner of an app, a resource allows full-access
   */
  | "owner";

type Acl = InferModel<typeof acls> & {
  access: AccessType;
  createdAt: Date;
  updatedAt?: Date | null;
  archivedAt?: Date | null;
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
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
          id: uniqueId(19),
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
