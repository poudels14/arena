import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import { EmbeddingsModel } from "@arena/cloud/llm";
import { Repo } from "./repo";

type Context = {
  dbpool: Pool;
  repo: Repo;
  user?: { id: string; email: string };
  llm: {
    embeddingsModel: EmbeddingsModel;
  };
};

const p = procedure<Context>().use(async ({ ctx, next }) => {
  if (process.env.DISABLE_AUTH == "true") {
    return await next({
      ctx: {
        ...ctx,
        user: {
          id: "1",
          email: "test-user@test.com",
        },
      },
    });
  }
  // TODO: parse use from header
  return await next({
    ctx,
  });
});

export { p };
export type { Context };
