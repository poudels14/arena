import { drizzle, jsonb, pgTable, timestamp, varchar } from "@arena/db/pg";
import { Status } from "@arena/sdk/plugins/workflow";
import { Context } from "./context";

const workflowRuns = pgTable("workflow_runs", {
  id: varchar("id").notNull(),
  workspaceId: varchar("workspace_id").notNull(),
  parentAppId: varchar("parent_app_id"),
  config: jsonb("config").notNull(),
  state: jsonb("state").notNull(),
  status: varchar("status").notNull(),
  template: jsonb("template").notNull(),
  triggeredBy: jsonb("triggered_by").notNull(),
  triggeredAt: timestamp("triggered_at").defaultNow(),
  lastHeartbeatAt: timestamp("last_heartbeat_at"),
});

type WorkflowRun = {
  id: string;
  workspaceId: string;
  parentAppId: string | null | undefined;
  config: any;
  state: {
    input?: any;
  };
  status: Status;
  template: any;
  triggeredBy: any;
  triggeredAt: Date;
  lastHeartbeatAt: Date | null;
};

const createRepo = (ctx: Context) => {
  const db = drizzle(ctx.client);
  return {
    async insertWorkflowRun(run: WorkflowRun): Promise<Required<WorkflowRun>> {
      const workspaceRows = await db
        .insert(workflowRuns)
        .values(run)
        .returning();

      return workspaceRows[0] as WorkflowRun;
    },
  };
};

export { workflowRuns, createRepo };
export type { WorkflowRun };
