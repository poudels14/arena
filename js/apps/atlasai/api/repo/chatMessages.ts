import { InferModel, eq } from "drizzle-orm";
import { jsonb, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

const chatMessages = pgTable("chat_messages", {
  id: varchar("id").notNull(),
  threadId: varchar("thread_id").notNull(),
  parentId: varchar("parent_id"),
  role: varchar("role").notNull(),
  userId: varchar("user_id"),
  message: jsonb("message").notNull(),
  metadata: jsonb("metadata").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
});

type ChatMessage = InferModel<typeof chatMessages> & {
  message: {
    content?: string | null;
  };
  metadata: any;
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(message: ChatMessage): Promise<void> {
      await db.insert(chatMessages).values(message);
    },
    async list(filter: { threadId: string }): Promise<ChatMessage[]> {
      const rows = await db
        .with()
        .select()
        .from(chatMessages)
        .where(eq(chatMessages.threadId, filter.threadId))
        .orderBy(chatMessages.createdAt);
      return rows as ChatMessage[];
    },
  };
};

export { createRepo };
export type { ChatMessage };
