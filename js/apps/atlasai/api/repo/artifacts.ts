import { InferModel, and, desc, eq, isNull } from "drizzle-orm";
import {
  integer,
  jsonb,
  pgTable,
  timestamp,
  varchar,
} from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

const artifacts = pgTable("chat_artifacts", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
  messageId: varchar("message_id").notNull(),
  threadId: varchar("thread_id").notNull(),
  size: integer("size").notNull(),
  file: jsonb("file").notNull(),
  metadata: jsonb("metadata").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type Artifact = InferModel<typeof artifacts> & {
  file: {
    // base64 encoded file content
    content: string;
  };
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(artifact: Omit<Artifact, "archivedAt">): Promise<void> {
      await db.insert(artifacts).values({ ...artifact, archivedAt: null });
    },
    async get(filter: { id: string }): Promise<Artifact> {
      const rows = await db
        .with()
        .select()
        .from(artifacts)
        .where(eq(artifacts.id, filter.id));
      return rows[0] as Artifact;
    },
    async list(
      filter: { threadId?: string },
      options: { includeContent?: boolean; limit: number }
    ): Promise<Artifact[]> {
      const rows = await db
        .with()
        .select({
          id: artifacts.id,
          name: artifacts.name,
          threadId: artifacts.threadId,
          messageId: artifacts.messageId,
          metadata: artifacts.metadata,
          createdAt: artifacts.createdAt,
          ...(options.includeContent
            ? {
                file: artifacts.file,
              }
            : {}),
        })
        .from(artifacts)
        .where(
          and(
            filter.threadId
              ? eq(artifacts.threadId, filter.threadId)
              : isNull(artifacts.archivedAt),
            isNull(artifacts.archivedAt)
          )
        )
        .orderBy(desc(artifacts.createdAt))
        .limit(options.limit);
      return rows as Artifact[];
    },
    async deleteByThreadId(filter: { threadId: string }) {
      await db.delete(artifacts).where(eq(artifacts.threadId, filter.threadId));
    },
  };
};

export { createRepo };
export type { Artifact };
