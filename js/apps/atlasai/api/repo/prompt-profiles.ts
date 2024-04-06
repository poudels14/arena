import { InferModel, desc, eq, isNull, and } from "drizzle-orm";
import {
  boolean,
  jsonb,
  pgTable,
  text,
  timestamp,
  varchar,
} from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";

const promptProfiles = pgTable("prompt_profiles", {
  id: varchar("id").notNull(),
  name: varchar("name").notNull(),
  description: text("description").notNull(),
  template: text("template").notNull(),
  bookmarked: boolean("bookmarked").notNull(),
  default: boolean("default").notNull(),
  metadata: jsonb("metadata").notNull(),
  createdAt: timestamp("created_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type PromptProfile = InferModel<typeof promptProfiles> & {};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(
      promptTemplate: Omit<PromptProfile, "archivedAt">
    ): Promise<void> {
      await db
        .insert(promptProfiles)
        .values({ ...promptTemplate, archivedAt: null });
    },
    async get(id: string): Promise<PromptProfile> {
      const rows = await db
        .with()
        .select()
        .from(promptProfiles)
        .where(eq(promptProfiles.id, id));
      return rows[0] as PromptProfile;
    },
    async list(
      filter: {
        bookmarked?: boolean;
        default?: boolean;
      },
      options: {
        includePrompt?: boolean;
      } = {}
    ): Promise<PromptProfile[]> {
      const conditions = [isNull(promptProfiles.archivedAt)];
      if (filter.bookmarked) {
        conditions.push(eq(promptProfiles.bookmarked, filter.bookmarked));
      }
      if (filter.default) {
        conditions.push(eq(promptProfiles.default, filter.default));
      }
      const rows = await db
        .with()
        .select({
          id: promptProfiles.id,
          name: promptProfiles.name,
          description: promptProfiles.description,
          bookmarked: promptProfiles.bookmarked,
          default: promptProfiles.default,
          metadata: promptProfiles.metadata,
          createdAt: promptProfiles.createdAt,
          ...(options.includePrompt
            ? {
                template: promptProfiles.template,
              }
            : {}),
        })
        .from(promptProfiles)
        .where(and(...conditions))
        .orderBy(desc(promptProfiles.createdAt));
      return rows as PromptProfile[];
    },
  };
};

export { createRepo };
export type { PromptProfile };
