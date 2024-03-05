import {
  InferModel,
  and,
  eq,
  gt,
  inArray,
  isNotNull,
  or,
  sql,
} from "drizzle-orm";
import { jsonb, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

export const embeddings = pgTable("file_embeddings", {
  id: varchar("id").notNull(),
  fileId: varchar("file_id").notNull(),
  directoryId: varchar("directory_id"),
  metadata: jsonb("metadata"),
  // embeddings VECTOR
  embeddings: jsonb("embeddings").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
});

type Embedding = InferModel<typeof embeddings> & {
  metadata: {
    // start index of the file chunk
    start: number;
    // end index of the file chunk
    end: number;
  } & Record<string, any>;
};
type EmbeddingWithScore = Embedding & { score: number };

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(embedding: Embedding): Promise<Embedding> {
      await db.insert(embeddings).values(embedding);
      return embedding as Embedding;
    },
    async search(options: {
      embeddings: number[];
      fileIds?: string[];
      // if passed, only searches the embeddings of the files that belong to
      // one of the given directory ids
      directories?: string[];
      // top K limit
      limit: number;
    }): Promise<
      Pick<EmbeddingWithScore, "id" | "fileId" | "metadata" | "score">[]
    > {
      const scoreExpr = sql.raw(
        "l2(embeddings, " + "'[" + options.embeddings.join(",") + "]') as score"
      );
      const rows = await db
        .select({
          id: embeddings.id,
          fileId: embeddings.fileId,
          metadata: embeddings.metadata,
          createdAt: embeddings.createdAt,
          score: scoreExpr,
        })
        .from(embeddings)
        .where(
          or(
            options.directories?.length
              ? inArray(embeddings.directoryId, options.directories)
              : isNotNull(embeddings.directoryId),
            options.fileIds
              ? inArray(embeddings.fileId, options.fileIds)
              : isNotNull(embeddings.directoryId)
          )
        )
        .orderBy(sql.raw("score DESC"))
        .limit(options.limit);

      return rows as EmbeddingWithScore[];
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
    async deleteByFileIds(fileIds: string[]) {
      await db.delete(embeddings).where(inArray(embeddings.fileId, fileIds));
    },
  };
};

export { createRepo };
export type { Embedding };
