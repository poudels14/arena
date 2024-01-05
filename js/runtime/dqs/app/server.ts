import { serve, mergedRouter } from "@arena/runtime/server";
// @ts-expect-error
import * as appTemplate from "@dqs/template/app";
import { DatabaseConfig } from "@portal/sdk/db";
import { router as adminRouter } from "./admin";
// import { router as workflowRouter } from "./workflow";
// @ts-expect-error
import { createDbClient } from "@arena/dqs/utils";

const router = mergedRouter({
  routers: [adminRouter, appTemplate.router],
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
    console.log(req);
    if (!state.dbClients) {
      state.dbClients = Object.fromEntries(
        await Promise.all(
          Object.entries(
            appTemplate.databases as Record<string, DatabaseConfig>
          ).map(async ([dbName, db]) => {
            return [
              dbName,
              await createDbClient({
                type: db.type,
                name: dbName,
                baseDir: "/db",
              }),
            ];
          })
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
