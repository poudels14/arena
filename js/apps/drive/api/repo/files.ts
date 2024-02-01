import { InferModel, and, eq, isNull } from "drizzle-orm";
import {
  jsonb,
  pgTable,
  text,
  timestamp,
  varchar,
  boolean,
  integer,
} from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

export const files = pgTable("files", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
  description: text("description"),
  parentId: varchar("parent_id"),
  isDirectory: boolean("is_directory"),
  size: integer("size").notNull(),
  metadata: jsonb("metadata"),
  file: jsonb("file"),
  createdBy: varchar("created_by"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type File = InferModel<typeof files>;

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(file: Omit<File, "updatedAt" | "archivedAt">): Promise<File> {
      file = {
        ...file,
        createdAt: new Date(),
        updatedAt: new Date(),
        archivedAt: null,
      } as File;
      await db.insert(files).values(file);
      return file as File;
    },
    async fetchById(id: string | null): Promise<File | null> {
      const rows = await db
        .select()
        .from(files)
        .where(
          and(
            id == null ? isNull(files.id) : eq(files.id, id),
            isNull(files.archivedAt)
          )
        );
      return (rows[0] || null) as File | null;
    },
    async listFiles(filters: {
      parentId: string | null;
    }): Promise<
      Pick<
        File,
        | "id"
        | "name"
        | "description"
        | "parentId"
        | "isDirectory"
        | "createdBy"
        | "createdAt"
        | "updatedAt"
      >[]
    > {
      const rows = await db
        .select({
          id: files.id,
          name: files.name,
          description: files.description,
          parentId: files.parentId,
          isDirectory: files.isDirectory,
          createdBy: files.createdBy,
          createdAt: files.createdAt,
          updatedAt: files.updatedAt,
        })
        .from(files)
        .where(
          and(
            filters.parentId == null
              ? isNull(files.parentId)
              : eq(files.parentId, filters.parentId),
            isNull(files.archivedAt)
          )
        );
      return rows as File[];
    },
    async archiveById(id: string): Promise<Pick<Required<File>, "archivedAt">> {
      throw new Error("not implemented");
      // TODO: archive all files and directories inside the given directory
      const archivedAt = new Date();
      await db
        .update(files)
        .set({
          archivedAt,
        })
        .where(and(eq(files.id, id), isNull(files.archivedAt)));
      return { archivedAt };
    },
  };
};

export { createRepo };
export type { File };
