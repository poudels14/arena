import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import { Repo } from "./repo";

type Context = {
  dbpool: Pool;
  repo: Repo;
  user?: { id: string };
};

const p = procedure<Context>().use(async ({ ctx, next }) => {
  return await next({
    ctx,
  });
  // TODO(sagar): do auth
});

export { p };
export type { Context };
