import { Pool } from "@arena/runtime/postgres";
import {
  PageEvent,
  chainMiddlewares,
  createHandler,
} from "@portal/server-core";
import { ServerRoot, renderToStringAsync } from "@portal/solidjs/server";
// import { createDefaultFileRouter } from "@portal/server-dev/solidjs";
import { Client as S3Client } from "@arena/cloud/s3";
import Root from "@portal/workspace/app/root";

import { router } from "~/api/index";
import { createRepo } from "./api/repo";
import { env } from "./api/env";

const dbpool = new Pool({
  host: env.DATABASE_HOST,
  port: env.DATABASE_PORT,
  database: env.DATABASE_NAME,
  user: env.DATABASE_USER,
  password: env.DATABASE_PASSWORD || "",
});

// let fileRouter: any;
// fileRouter = await createDefaultFileRouter({
//   baseDir: process.cwd(),
//   env: {
//     NODE_ENV: "development",
//     SSR: "false",
//     PORTAL_SSR: "false",
//     PORTAL_ENTRY_CLIENT: "./entry-client.tsx",
//   },
//   babel: {},
//   resolverConfig: {
//     preserveSymlink: true,
//     alias: {
//       "~": "./app",
//     },
//     conditions: ["solid", "browser"],
//     dedupe: [
//       "solid-js",
//       "@solidjs/router",
//       "@solidjs/meta",
//       "@arena/core",
//       "@portal/solid-store",
//       "@portal/solid-router",
//       "@portal/solid-query",
//       "@portal/solidjs",
//     ],
//   },
//   transpilerConfig: {
//     resolveImports: true,
//   },
// });

const handler = chainMiddlewares<{ event: PageEvent }>(
  // async ({ event }) => {
  //   if (process.env.NODE_ENV == "development") {
  //     const res = await fileRouter.route(event.request);
  //     console.log("filerouter res =", res);
  //     if (res && res.status != 404) {
  //       return res;
  //     }
  //   }
  // },
  async ({ event }) => {
    const repo = await createRepo({ pool: dbpool });
    try {
      const result = await router.route(event.request, {
        env: process.env,
        context: {
          host: env.HOST,
          env,
          dbpool,
          repo,
          user: null,
          s3Client: new S3Client({
            region: {
              Custom: {
                region: "none",
                endpoint: env.S3_ENDPOINT,
              },
            },
            credentials: {
              access_key: env.S3_ACCESS_KEY,
              secret_key: env.S3_ACCESS_SECRET,
            },
            withPathStyle: true,
          }),
        },
      });
      if (result?.status == 404) {
        return;
      }
      return result;
    } catch (e) {
      console.error(e);
      return new Response("Internal Server Error", { status: 500 });
    } finally {
      await repo.release();
    }
  },
  renderToStringAsync(({ event }) => {
    return <ServerRoot event={event} Root={Root} />;
  })
);

const { fetch } = createHandler(async (event) => await handler({ event }));

export { fetch };
export default { fetch };
