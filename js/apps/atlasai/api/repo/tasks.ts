import { InferModel, and, desc, eq, isNotNull } from "drizzle-orm";
import { jsonb, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { pick } from "lodash-es";

const taskExecutions = pgTable("task_executions", {
  id: varchar("id").notNull(),
  taskId: varchar("task_id").notNull(),
  threadId: varchar("thread_id").notNull(),
  messageId: varchar("message_id"),
  status: varchar("status").notNull(),
  metadata: jsonb("metadata").notNull(),
  state: jsonb("state").notNull(),
  startedAt: timestamp("started_at").defaultNow(),
});

type DbTaskExecution = InferModel<typeof taskExecutions> & {
  status: "STARTED" | "TERMINATED" | "COMPLETED" | "ERROR";
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(
      taskExecution: Pick<
        DbTaskExecution,
        "id" | "taskId" | "threadId" | "messageId" | "metadata" | "state"
      >
    ): Promise<void> {
      await db.insert(taskExecutions).values({
        ...taskExecution,
        status: "STARTED",
        startedAt: new Date(),
      });
    },
    async update(
      taskExecution: Partial<
        Pick<DbTaskExecution, "status" | "metadata" | "state">
      > &
        Pick<DbTaskExecution, "id">
    ): Promise<void> {
      await db
        .update(taskExecutions)
        .set(pick(taskExecution, "status", "metadata", "state"))
        .where(eq(taskExecutions.id, taskExecution.id));
    },
    async getById(id: string): Promise<DbTaskExecution | undefined> {
      const rows = await db
        .select()
        .from(taskExecutions)
        .where(eq(taskExecutions.id, id));
      return rows[0] as DbTaskExecution;
    },
    async list(filter: {
      threadId?: string;
      status?: string;
    }): Promise<DbTaskExecution[]> {
      const rows = await db
        .select()
        .from(taskExecutions)
        .where(
          and(
            filter.threadId
              ? eq(taskExecutions.threadId, filter.threadId)
              : isNotNull(taskExecutions.id),
            filter.status
              ? eq(taskExecutions.status, filter.status)
              : isNotNull(taskExecutions.id)
          )
        )
        .orderBy(desc(taskExecutions.startedAt));
      return rows as DbTaskExecution[];
    },
  };
};

export { createRepo };
export type { DbTaskExecution };
