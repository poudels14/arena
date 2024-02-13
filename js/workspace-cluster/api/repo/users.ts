import { and, eq, isNull, InferModel } from "drizzle-orm";
import { jsonb, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { PostgresJsDatabase } from "drizzle-orm/postgres-js";
import { merge } from "lodash-es";

export const users = pgTable("users", {
  id: varchar("id").notNull(),
  email: varchar("email").notNull(),
  firstName: varchar("first_name"),
  lastName: varchar("last_name"),
  config: jsonb("config"),
  createdAt: timestamp("created_at").defaultNow(),
  archivedAt: timestamp("archived_at"),
});

type User = InferModel<typeof users> & {
  archivedAt?: Date | null;
  config: {
    waitlisted?: boolean;
  };
};

const createRepo = (db: PostgresJsDatabase<Record<string, never>>) => {
  return {
    async insert(
      user: Partial<User> & Pick<User, "id" | "email">
    ): Promise<User> {
      const dbUser = {
        config: { waitlisted: true },
        createdAt: new Date(),
        archivedAt: null,
        ...user,
      };
      await db.insert(users).values(dbUser);
      return withDefaultUserConfig(dbUser as User)!;
    },
    async fetchById(id: string): Promise<User | null> {
      const rows = await db
        .select()
        .from(users)
        .where(and(eq(users.id, id), isNull(users.archivedAt)));
      return withDefaultUserConfig(rows[0] as User);
    },
    async fetchByEmail(email: string): Promise<User | null> {
      const rows = await db
        .select()
        .from(users)
        .where(and(eq(users.email, email), isNull(users.archivedAt)));
      return withDefaultUserConfig(rows[0] as User);
    },
  };
};

const withDefaultUserConfig = (user: User | undefined) => {
  if (!user) return null;
  return {
    ...user,
    config: merge(
      {
        // Note(sp): set waitlisted by default
        waitlisted: true,
      },
      user.config
    ),
  };
};

export { createRepo };
export type { User };
