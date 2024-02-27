import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import { Repo } from "./repo";
import { Env } from "./env";

type Context = {
  env: Env;
  dbpool: Pool;
  repo: Repo;
  user?: { id: string };
};

const p = procedure<Context>().use(async ({ ctx, req, next }) => {
  const portalUser = req.headers.get("x-portal-user");
  const user = JSON.parse(portalUser || "null");
  return await next({
    ctx: {
      ...ctx,
      user,
    },
  });
});

export { p };
export type { Context };
