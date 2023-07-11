import { createRouter, procedure } from "@arena/runtime/server";
import { setup as setupApp, DatabaseClient } from "@arena/sdk/apps";
// @ts-expect-error
import { migrations } from "@app/template";

const p = procedure<{ db: DatabaseClient }>();
const router = createRouter({
  routes: {
    "/_healthy": p.query(async ({ ctx }) => {
      return "Ok";
    }),
    "/_setup": p.mutate(async ({ ctx }) => {
      try {
        await setupApp({
          client: ctx.db,
          migrations,
        });
      } catch (e: any) {
        return { error: e.message };
      }

      return { success: "true" };
    }),
  },
});

export { router };
