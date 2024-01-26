import { InferModel, and, eq, isNull } from "drizzle-orm";
import { pgTable, text, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

export const appTemplates = pgTable("app_templates", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
  description: text("description"),
  defaultVersion: varchar("default_version"),
  ownerId: varchar("owner_id").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type AppTemplate = InferModel<typeof appTemplates>;

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(
      appTemplate: Omit<AppTemplate, "createdAt" | "archivedAt">
    ): Promise<AppTemplate> {
      appTemplate = {
        ...appTemplate,
        createdAt: new Date(),
        archivedAt: null,
      } as AppTemplate;
      await db.insert(appTemplates).values(appTemplate);
      return appTemplate as AppTemplate;
    },
    async fetchById(id: string): Promise<AppTemplate | null> {
      const rows = await db
        .select()
        .from(appTemplates)
        .where(and(isNull(appTemplates.archivedAt), eq(appTemplates.id, id)));
      return (rows[0] || null) as AppTemplate | null;
    },
    async archiveById(
      id: string
    ): Promise<Pick<Required<AppTemplate>, "archivedAt">> {
      const archivedAt = new Date();
      await db
        .update(appTemplates)
        .set({
          archivedAt,
        })
        .where(and(eq(appTemplates.id, id), isNull(appTemplates.archivedAt)));
      return { archivedAt };
    },
  };
};

export { createRepo };
export type { AppTemplate };
