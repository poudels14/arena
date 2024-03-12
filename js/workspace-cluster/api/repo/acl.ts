import { InferModel, and, eq, isNull, or } from "drizzle-orm";
import { jsonb, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { z } from "zod";

export const acls = pgTable("acls", {
  id: varchar("id").notNull(),
  workspaceId: varchar("workspace_id").notNull(),
  /**
   * Special user ids:
   *
   * - "everyone": everyone in the workspace
   * - "public": shared publicly
   */
  userId: varchar("user_id").notNull(),
  accessGroup: varchar("access_group"),
  appId: varchar("app_id"),
  appTemplateId: varchar("app_template_id"),
  metadata: jsonb("metadata").notNull(),
  resourceId: varchar("resource_id"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

const aclCommand = z.enum(["*", "SELECT", "INSERT", "UPDATE", "DELETE"]);
type AclCommand = z.infer<typeof aclCommand>;

type Acl = InferModel<typeof acls> & {
  accessGroup: string;
  metadata: {
    filters: {
      command: AclCommand;
      // table name
      table: string;
      // SQL query filter; eg: `id = 1`, `id > 10`, etc
      condition: string;
    }[];
    // list of entities that this acl provides access to
    // this is mostly used by the apps to keep track of shared
    // resources in case they need to get a list of the shared
    // resources; for example, when sharing files, might need to
    // list shared files/directories
    entities?: { id: string }[];
  };
  createdAt: Date;
  updatedAt?: Date | null;
  archivedAt?: Date | null;
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async listAccess(filter: {
      userId?: string;
      workspaceId?: string | undefined;
      appId?: string;
      appTemplateId?: string;
    }): Promise<Required<Acl>[]> {
      const conditions = [isNull(acls.archivedAt)];

      if (filter.userId) {
        conditions.push(
          or(eq(acls.userId, filter.userId), eq(acls.userId, "public"))!
        );
      }

      if (filter.workspaceId) {
        conditions.push(eq(acls.workspaceId, filter.workspaceId));
      }
      if (filter.appId) {
        conditions.push(eq(acls.appId, filter.appId));
      }
      if (filter.appTemplateId) {
        conditions.push(eq(acls.appTemplateId, filter.appTemplateId));
      }
      const rows = await db
        .select()
        .from(acls)
        .where(and(...conditions));
      return rows as Acl[];
    },
    async getById(id: string): Promise<Acl | undefined> {
      const rows = await db
        .select()
        .from(acls)
        .where(and(eq(acls.id, id), isNull(acls.archivedAt)));
      return rows[0] as Acl | undefined;
    },
    async addAccess(
      acl: Pick<
        Acl,
        "id" | "workspaceId" | "userId" | "accessGroup" | "metadata"
      > &
        Partial<
          Pick<Acl, "appId" | "appTemplateId" | "metadata" | "resourceId">
        >
    ): Promise<Pick<Acl, "id" | "createdAt">> {
      const createdAt = new Date();
      await db.insert(acls).values({
        ...acl,
        createdAt,
      });
      return { ...acl, createdAt } as Pick<Acl, "id" | "createdAt">;
    },
    async archiveAccess(id: string) {
      await db
        .update(acls)
        .set({
          archivedAt: new Date(),
        })
        .where(eq(acls.id, id));
    },
  };
};

export { createRepo, aclCommand };
export type { Acl };
