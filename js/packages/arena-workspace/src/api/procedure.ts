import { procedure } from "@arena/runtime/server";
import { DatabaseClients } from "@arena/sdk/db";
import { databases } from "../../server";

type Context = {
  user: any;
  dbs: DatabaseClients<typeof databases>;
};

const p = procedure<Context>().use(async ({ ctx, next }) => {
  return await next({
    ctx,
  });
  // TODO(sagar): do auth
});

export { p };
export type { Context };
