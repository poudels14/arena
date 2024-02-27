import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import { EmbeddingsModel } from "@arena/cloud/llm";
import { Repo } from "./repo";

type Context = {
  dbpool: Pool;
  repo: Repo;
  user: { id: string; email?: string };
  app: { id: string; template: { id: string } };
  llm: {
    embeddingsModel: EmbeddingsModel;
  };
};

const p = procedure<Context>().use(async ({ ctx, req, next }) => {
  const portalUser = req.headers.get("x-portal-user");
  const portalApp = req.headers.get("x-portal-app");
  const user = JSON.parse(portalUser || "null");
  const app = JSON.parse(portalApp || "null");
  return await next({
    ctx: {
      ...ctx,
      user,
      app,
    },
  });
});

export { p };
export type { Context };
