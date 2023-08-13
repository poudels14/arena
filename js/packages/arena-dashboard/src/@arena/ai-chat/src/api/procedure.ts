import { procedure } from "@arena/runtime/server";
import { DatabaseClients } from "@arena/sdk/db";
import { databases } from "../../server";

const p = procedure<{
  user: any;
  dbs: DatabaseClients<typeof databases>;
}>().use(async ({ ctx, next }) => {
  return await next({ ctx });
  // TODO(sagar): do auth
});

export { p };
