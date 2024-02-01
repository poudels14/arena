import { InferModel, and, eq, isNotNull, sql } from "drizzle-orm";
import { jsonb, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

export const embeddings = pgTable("file_embeddings", {
  id: varchar("id").notNull(),
  fileId: varchar("file_id").notNull(),
  metadata: jsonb("metadata"),
  // embeddings VECTOR
  embeddings: jsonb("embeddings").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
});

type Embedding = InferModel<typeof embeddings>;

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(embedding: Embedding): Promise<Embedding> {
      await db.insert(embeddings).values(embedding);
      return embedding as Embedding;
    },
    async search(options: {
      embeddings: number[];
      // top K limit
      limit: number;
    }): Promise<Pick<Embedding, "id" | "fileId" | "metadata">[]> {
      const rows = await db
        .select({
          id: embeddings.id,
          fileId: embeddings.fileId,
          metadata: embeddings.metadata,
          createdAt: embeddings.createdAt,
        })
        .from(embeddings)
        .orderBy(
          sql.raw(
            "l2(embeddings, " + "'[" + options.embeddings.join(",") + "]') DESC"
          )
        )
        .limit(options.limit);

      return rows as Embedding[];
    },
    async list(filters: {
      fileId?: string;
    }): Promise<Pick<Embedding, "id" | "fileId" | "metadata">[]> {
      const rows = await db
        .select({
          id: embeddings.id,
          fileId: embeddings.fileId,
          embeddings: embeddings.embeddings,
          metadata: embeddings.metadata,
          createdAt: embeddings.createdAt,
        })
        .from(embeddings)
        .where(
          and(
            filters.fileId
              ? eq(embeddings.fileId, filters.fileId)
              : isNotNull(embeddings.fileId)
          )
        );
      return rows as Embedding[];
    },
  };
};

export { createRepo };
export type { Embedding };
