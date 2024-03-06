import { InferModel, and, eq, isNull, or } from "drizzle-orm";
import { jsonb, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

export const settings = pgTable("settings", {
  id: varchar("id").notNull(),
  workspaceId: varchar("workspace_id"),
  userId: varchar("user_id"),
  namespace: varchar("namespace"),
  metadata: jsonb("metadata"),
  createdAt: timestamp("created_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type Setting = InferModel<typeof settings> & {
  metadata: any;
  archivedAt?: Date | null;
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(
      setting: Omit<Setting, "createdAt" | "archivedAt">
    ): Promise<Setting> {
      setting = {
        ...setting,
        createdAt: new Date(),
        archivedAt: null,
      } as Setting;
      await db.insert(settings).values(setting);
      return setting as Setting;
    },
    async getById(id: string): Promise<Setting | null> {
      const rows = await db
        .select()
        .from(settings)
        .where(and(eq(settings.id, id), isNull(settings.archivedAt)));
      return (rows[0] as Setting) || null;
    },
    async list(filter: {
      id?: string;
      workspaceId?: string;
      userId?: string;
      namespace?: string;
    }): Promise<Setting[]> {
      if (!filter.workspaceId && !filter.userId) {
        return [];
      }
      const conditions = [];
      if (filter.id) {
        conditions.push(eq(settings.id, filter.id));
      }
      if (filter.workspaceId) {
        conditions.push(
          or(
            eq(settings.workspaceId, filter.workspaceId),
            isNull(settings.workspaceId)
          )
        );
      }
      if (filter.userId) {
        conditions.push(
          or(eq(settings.userId, filter.userId), isNull(settings.userId))
        );
      }
      if (filter.namespace) {
        conditions.push(eq(settings.namespace, filter.namespace));
      }
      const rows = await db
        .select()
        .from(settings)
        .where(and(...conditions, isNull(settings.archivedAt)));
      return rows as Setting[];
    },
    async listGlobalSetting(filter: {
      namespace?: string;
    }): Promise<Setting[]> {
      const conditions = [];
      if (filter.namespace) {
        conditions.push(eq(settings.namespace, filter.namespace));
      }
      const rows = await db
        .select()
        .from(settings)
        .where(
          and(
            ...conditions,
            isNull(settings.userId),
            isNull(settings.workspaceId),
            isNull(settings.archivedAt)
          )
        );
      return rows as Setting[];
    },
    async updateById(
      id: string,
      metadata: Pick<Setting, "metadata">
    ): Promise<void> {
      await db
        .update(settings)
        .set({
          metadata,
        })
        .where(and(eq(settings.id, id), isNull(settings.archivedAt)));
    },
    async archiveById(
      id: string
    ): Promise<Pick<Required<Setting>, "archivedAt">> {
      const archivedAt = new Date();
      await db
        .update(settings)
        .set({
          archivedAt,
        })
        .where(and(eq(settings.id, id), isNull(settings.archivedAt)));
      return { archivedAt };
    },
  };
};

export { createRepo };
export type { Setting };
