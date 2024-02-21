import { InferModel, and, eq, inArray, isNull } from "drizzle-orm";
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
  contentType: varchar("content_type"),
  createdBy: varchar("created_by"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type File = InferModel<typeof files> & {
  file: {
    content: string;
  } | null;
};

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
    async fetchFileNamesByIds(
      ids: string[]
    ): Promise<Pick<File, "id" | "name">[]> {
      const rows = await db
        .select({
          id: files.id,
          name: files.name,
        })
        .from(files)
        .where(and(inArray(files.id, ids), isNull(files.archivedAt)));
      return rows as File[];
    },
    async fetchFileContent(
      ids: string[]
    ): Promise<Pick<File, "id" | "name" | "parentId" | "file">[]> {
      const rows = await db
        .select({
          id: files.id,
          name: files.name,
          parentId: files.parentId,
          file: files.file,
        })
        .from(files)
        .where(and(inArray(files.id, ids), isNull(files.archivedAt)));
      return rows as File[];
    },
    // This list all the files and directories of the given parent file id
    // if there are derived files like pdf will have one (plain text version),
    // this can be used to get derived file ids of the original file
    async fetchDirectChildren(filters: {
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
    // breadcrumb of a directory will also include itself
    async getBreadcrumb(filters: {
      directoryId: string | null;
    }): Promise<Pick<File, "id" | "name" | "description" | "parentId">[]> {
      // root directory (id = null) doesn't have breadcrumb
      if (filters.directoryId == null) {
        return [];
      }
      // Note: since arenasql doesn't support recursive CTE, do recursion here
      const getDirectory = async (directoryId: string) => {
        const rows = await db
          .select({
            id: files.id,
            name: files.name,
            description: files.description,
            parentId: files.parentId,
          })
          .from(files)
          .where(
            and(
              eq(files.id, directoryId),
              eq(files.isDirectory, true),
              isNull(files.archivedAt)
            )
          );
        return rows.length > 0 ? rows[0] : null;
      };

      const breadcrumbs = [];
      let directoryId: string | null = filters.directoryId;
      while (directoryId) {
        const directory = await getDirectory(directoryId);
        if (directory) {
          breadcrumbs.unshift(directory);
          directoryId = directory.parentId;
        } else {
          return breadcrumbs;
        }
      }
      return breadcrumbs as File[];
    },
    // list all the nested directories inside the given directory
    async listAllSubDirectories(filters: {
      parentId: string | null;
    }): Promise<Pick<File, "id" | "name" | "description" | "parentId">[]> {
      // Note: since arenasql doesn't support recursive CTE, do recursion here
      const listDirectories = async (directoryId: string | null) => {
        return await db
          .select({
            id: files.id,
            name: files.name,
            description: files.description,
            parentId: files.parentId,
          })
          .from(files)
          .where(
            and(
              directoryId == null
                ? isNull(files.parentId)
                : eq(files.parentId, directoryId),
              eq(files.isDirectory, true),
              isNull(files.archivedAt)
            )
          );
      };

      const allDirectories = [];
      const stack = [filters.parentId];
      while (stack.length > 0) {
        const dirs = await listDirectories(stack.pop()!);
        allDirectories.push(...dirs);
        stack.push(...dirs.map((dir) => dir.id));
      }
      return allDirectories as File[];
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
