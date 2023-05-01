import { sql } from "@arena/slonik";
import { Context } from "./context";

type App = {
  id: string;
  name: string;
  description?: string;
  workspaceId: string;
  createdAt: string | null;
  archivedAt?: string | null;
};

const createRepo = (ctx: Context) => {
  return {
    async fetchById(id: string): Promise<App | null> {
      const { rows } = await ctx.client.query<App>(
        sql`SELECT * FROM apps WHERE archived_at IS NULL AND id = ${id}`
      );
      return rows?.[0];
    },
  };
};

export { createRepo };
export type { App };
