import {
  InferModel,
  and,
  drizzle,
  eq,
  isNull,
  jsonb,
  pgTable,
  sql,
  text,
  timestamp,
  varchar,
} from "@arena/db/pg";
import { Context } from "./context";
import { merge } from "lodash-es";

export const apps = pgTable("apps", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
  description: text("description"),
  template: jsonb("template"),
  workspaceId: varchar("workspace_id"),
  config: jsonb("config"),
  createdBy: varchar("created_by"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type App = InferModel<typeof apps> & {
  template: { id: string; version: string } | null;
  description?: string;
  config: any;
  archivedAt?: Date | null;
};

const createRepo = (ctx: Context) => {
  const db = drizzle(ctx.client);
  return {
    async insert(
      app: Omit<
        InferModel<typeof apps>,
        "createdAt" | "updatedAt" | "archivedAt"
      >
    ): Promise<App> {
      const rows = await db.insert(apps).values(app).returning({
        createdAt: apps.createdAt,
        updatedAt: apps.updatedAt,
        archivedAt: apps.archivedAt,
      });
      const updated = rows[0];
      return merge(app, updated) as App;
    },
    async fetchById(id: string): Promise<App | null> {
      const { rows } = await ctx.client.query<App>(
        sql`SELECT * FROM apps WHERE archived_at IS NULL AND id = ${id}`
      );
      return rows?.[0];
    },
    async listApps(filter: { workspaceId: string }): Promise<Required<App>[]> {
      const rows = await db
        .select()
        .from(apps)
        .where(
          and(eq(apps.workspaceId, filter.workspaceId), isNull(apps.archivedAt))
        );
      return rows as App[];
    },
    async archiveById(id: string): Promise<Pick<Required<App>, "archivedAt">> {
      const rows = await db
        .update(apps)
        .set({
          archivedAt: sql.raw(`NOW()`),
        })
        .where(and(eq(apps.id, id), isNull(apps.archivedAt)))
        .returning({
          archivedAt: apps.archivedAt,
        });
      return rows[0];
    },
  };
};

export { createRepo };
export type { App };
