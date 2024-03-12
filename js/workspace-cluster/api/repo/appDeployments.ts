import { InferModel, eq } from "drizzle-orm";
import { pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

export const appDeployments = pgTable("app_deployments", {
  id: varchar("id").notNull(),
  nodeId: varchar("node_id").notNull(),
  workspaceId: varchar("workspace_id").notNull(),
  appId: varchar("app_id"),
  rebootTriggeredAt: timestamp("reboot_triggered_at"),
});

type AppDeployment = InferModel<typeof appDeployments>;

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async reboot(filter: { appId: string }): Promise<void> {
      await db
        .update(appDeployments)
        .set({
          rebootTriggeredAt: new Date(),
        })
        .where(eq(appDeployments.appId, filter.appId));
    },
  };
};

export { createRepo };
export type { AppDeployment };
