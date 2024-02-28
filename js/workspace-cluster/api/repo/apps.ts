import { Client } from "@arena/runtime/postgres";
import { InferModel, and, eq, isNull } from "drizzle-orm";
import { jsonb, pgTable, text, timestamp, varchar } from "drizzle-orm/pg-core";
import { drizzle } from "drizzle-orm/postgres-js";

export const apps = pgTable("apps", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
  slug: varchar("slug").notNull(),
  description: text("description"),
  template: jsonb("template"),
  workspaceId: varchar("workspace_id").notNull(),
  ownerId: varchar("owner_id"),
  config: jsonb("config"),
  createdBy: varchar("created_by"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type App = InferModel<typeof apps> & {
  template: { id: string; version: string } | null;
  description?: string;
  config: {};
  archivedAt?: Date | null;
};

const createRepo = (client: Client) => {
  const db = drizzle(client);
  return {
    async insert(
      app: Omit<App, "createdAt" | "updatedAt" | "archivedAt">
    ): Promise<App> {
      app = {
        ...app,
        createdAt: new Date(),
        updatedAt: new Date(),
        archivedAt: null,
      } as App;
      await db.insert(apps).values(app);
      return app as App;
    },
    async fetchById(id: string): Promise<App | null> {
      const rows = await db
        .select()
        .from(apps)
        .where(and(isNull(apps.archivedAt), eq(apps.id, id)));
      return (rows[0] || null) as App | null;
    },
    async listApps(filters: {
      workspaceId: string;
      slug?: string;
    }): Promise<Required<App>[]> {
      const rows = await db
        .select()
        .from(apps)
        .where(
          and(
            eq(apps.workspaceId, filters.workspaceId),
            filters.slug
              ? eq(apps.slug, filters.slug)
              : isNull(apps.archivedAt),
            isNull(apps.archivedAt)
          )
        );
      return rows as App[];
    },
    async archiveById(id: string): Promise<Pick<Required<App>, "archivedAt">> {
      const archivedAt = new Date();
      await db
        .update(apps)
        .set({
          archivedAt,
        })
        .where(and(eq(apps.id, id), isNull(apps.archivedAt)));
      return { archivedAt };
    },
  };
};

export { createRepo };
export type { App };
