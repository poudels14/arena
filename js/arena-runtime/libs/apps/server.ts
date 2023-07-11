import { serve, mergedRouter } from "@arena/runtime/server";
// @ts-expect-error
import { router as appRouter } from "@app/template";
import { router as adminRouter } from "./routes";
import { createDbClient } from "./database";

const router = mergedRouter({
  routers: [adminRouter, appRouter],
});

let state: any = {
  dbClient: null,
};

serve({
  async fetch(req) {
    if (!state.dbClient) {
      state.dbClient = await createDbClient({
        path: "/db/data/main.sqlite",
      });
    }

    const res = await router.route(req, {
      context: {
        user: null,
        db: state.dbClient,
      },
      env: {},
    });

    if (res) {
      return res;
    }
    return new Response("Not found", {
      status: 404,
    });
  },
});
