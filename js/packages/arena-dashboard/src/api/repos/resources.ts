import {
  pgTable,
  varchar,
  text,
  InferModel,
  boolean,
  jsonb,
  timestamp,
  drizzle,
  eq,
  and,
  isNull,
  sql,
} from "@arena/db/pg";
import { merge } from "lodash-es";
import { Context } from "./context";

export const resources = pgTable("resources", {
  id: varchar("id"),
  workspaceId: varchar("workspace_id"),
  name: varchar("name").notNull(),
  description: text("description"),
  type: varchar("type").notNull(),
  secret: boolean("secret").notNull(),
  key: varchar("key"),
  value: jsonb("value").notNull(),
  contextId: varchar("context_id"),
  createdBy: varchar("created_by"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type DbResource = InferModel<typeof resources>;

const createRepo = (ctx: Context) => {
  const db = drizzle(ctx.client);
  return {
    async insert(
      config: Omit<DbResource, "createdAt" | "updatedAt" | "archivedAt">
    ): Promise<DbResource> {
      const rows = await db.insert(resources).values(config).returning({
        createdAt: resources.createdAt,
        updatedAt: resources.updatedAt,
        archivedAt: resources.archivedAt,
      });
      const updated = rows[0];
      return merge(config, updated) as DbResource;
    },
    async fetchByWorkspaceId(
      workspaceId: NonNullable<DbResource["workspaceId"]>
    ): Promise<Required<DbResource>[]> {
      const rows = await db
        .select()
        .from(resources)
        .where(
          and(
            eq(resources.workspaceId, workspaceId),
            isNull(resources.archivedAt)
          )
        );
      return rows;
    },
    async fetchById(id: string): Promise<DbResource | undefined> {
      const rows = await db
        .select()
        .from(resources)
        .where(and(eq(resources.id, id), isNull(resources.archivedAt)));
      return rows?.[0];
    },
    async archiveById(
      id: string
    ): Promise<Pick<Required<DbResource>, "archivedAt">> {
      const rows = await db
        .update(resources)
        .set({
          archivedAt: sql.raw(`NOW()`),
        })
        .where(and(eq(resources.id, id), isNull(resources.archivedAt)))
        .returning({
          archivedAt: resources.archivedAt,
        });
      return rows[0];
    },
  };
};

export { createRepo };
export type { DbResource };
