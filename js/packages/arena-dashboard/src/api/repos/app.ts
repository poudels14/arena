import {
  InferModel,
  and,
  drizzle,
  eq,
  isNull,
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
  workspaceId: varchar("workspace_id"),
  ownerId: varchar("owner_id"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type App = InferModel<typeof apps> & {
  description?: string;
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
    async fetch(filter: {
      workspaceId: string;
      ownerId: string;
    }): Promise<Required<App>[]> {
      const rows = await db
        .select()
        .from(apps)
        .where(
          and(
            eq(apps.workspaceId, filter.workspaceId),
            eq(apps.ownerId, filter.ownerId),
            isNull(apps.archivedAt)
          )
        );
      return rows as App[];
    },
    async fetchByOwnerId(ownerId: string): Promise<App[]> {
      const { rows } = await ctx.client.query<App>(
        sql`SELECT * FROM apps WHERE archived_at IS NULL AND owner_id = ${ownerId}`
      );
      return rows;
    },
  };
};

export { createRepo };
export type { App };
