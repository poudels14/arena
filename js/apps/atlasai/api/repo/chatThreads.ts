import { InferModel, desc, eq } from "drizzle-orm";
import { json, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { ChatThread } from "../chat/types";

const chatThreads = pgTable("chat_threads", {
  id: varchar("id").notNull(),
  title: varchar("title").notNull(),
  blockedBy: varchar("blocked_by"),
  metadata: json("metadata").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
});

type DbChatThread = InferModel<typeof chatThreads> & ChatThread;

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(thread: DbChatThread): Promise<void> {
      await db.insert(chatThreads).values(thread);
    },
    async update(
      thread: Partial<DbChatThread> & Pick<DbChatThread, "id">
    ): Promise<void> {
      await db
        .update(chatThreads)
        .set(thread)
        .where(eq(chatThreads.id, thread.id));
    },
    async getById(id: string): Promise<DbChatThread | undefined> {
      const rows = await db
        .select()
        .from(chatThreads)
        .where(eq(chatThreads.id, id));
      return rows[0] as DbChatThread;
    },
    async list(): Promise<DbChatThread[]> {
      const rows = await db
        .with()
        .select({
          id: chatThreads.id,
          title: chatThreads.title,
          blockedBy: chatThreads.blockedBy,
          metadata: chatThreads.metadata,
          createdAt: chatThreads.createdAt,
        })
        .from(chatThreads)
        .orderBy(desc(chatThreads.createdAt));

      return rows as DbChatThread[];
    },
    async deleteById(id: DbChatThread["id"]): Promise<void> {
      await db.delete(chatThreads).where(eq(chatThreads.id, id));
    },
  };
};

export { createRepo };
export type { ChatThread };
