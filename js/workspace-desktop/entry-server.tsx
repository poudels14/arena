import { Pool } from "@arena/runtime/postgres";
import {
  PageEvent,
  chainMiddlewares,
  createHandler,
} from "@portal/server-core";
import { ServerRoot, renderToStringAsync } from "@portal/solidjs/server";
import Root from "@portal/workspace/app/root";

import { env } from "@portal/workspace-cluster/api/utils/env";
import { router } from "@portal/workspace-cluster/api/desktop";
import { createRepo } from "@portal/workspace-cluster/api/repo";

const dbpool = new Pool({
  host: env.DATABASE_HOST,
  port: env.DATABASE_PORT,
  database: env.DATABASE_NAME,
  user: env.DATABASE_USER,
  password: env.DATABASE_PASSWORD || "",
  max: 100,
});

const handler = chainMiddlewares<{ event: PageEvent }>(
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
        },
      });

      // If the status code is 404 and the path isn't related to API or
      // registry, return undefined such that HTML renderer handles the
      // request
      if (result?.status == 404) {
        const url = new URL(event.request.url);
        if (!url.pathname.startsWith("/api")) return;
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
