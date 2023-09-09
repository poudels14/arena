import { serve, mergedRouter } from "@arena/runtime/server";
// @ts-expect-error
import { router as appRouter, databases } from "@dqs/app/template";
import { DatabaseConfig } from "@arena/sdk/db";
import { router as adminRouter } from "./routes";
import { createDbClient } from "./database";

const router = mergedRouter({
  routers: [adminRouter, appRouter],
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
});

let state: any = {
  dbClients: null,
};

serve({
  async fetch(req) {
    if (!state.dbClients) {
      state.dbClients = Object.fromEntries(
        await Promise.all(
          Object.entries(databases as Record<string, DatabaseConfig>).map(
            async ([dbName, db]) => {
              return [
                dbName,
                await createDbClient({
                  type: db.type,
                  name: dbName,
                  baseDir: "/db",
                }),
              ];
            }
          )
        )
      );
    }

    const res = await router.route(req, {
      context: {
        // TODO(sagar): set user
        user: null,
        dbs: state.dbClients,
      },
      env: process.env,
    });

    if (res) {
      return res;
    }
    return new Response("Not found", {
      status: 404,
    });
  },
});
