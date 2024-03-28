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
  createdAt: timestamp("created_at").notNull().defaultNow(),
});

type ChatMessage = InferModel<typeof chatMessages> & {
  message: {
    content?: string | MessageContent[];
  };
  metadata: any;
};

type MessageContent =
  | {
      type: "text";
      text: string;
    }
  | {
      type: "image_url";
      // `data:image/png;base64,...`
      image_url: string;
    };

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(message: ChatMessage): Promise<void> {
      await db.insert(chatMessages).values(message);
    },
    async list(filter: { threadId: string }): Promise<ChatMessage[]> {
      const rows = await db
        .select()
        .from(chatMessages)
        .where(eq(chatMessages.threadId, filter.threadId))
        .orderBy(chatMessages.createdAt);
      return rows as ChatMessage[];
    },
    async deleteByThreadId(filter: { threadId: string }) {
      await db
        .delete(chatMessages)
        .where(eq(chatMessages.threadId, filter.threadId));
    },
  };
};

export { createRepo };
export type { ChatMessage };
