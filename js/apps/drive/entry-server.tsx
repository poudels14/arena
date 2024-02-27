import { Pool } from "builtin://@arena/dqs/postgres";
import {
  PageEvent,
  chainMiddlewares,
  createHandler,
} from "@portal/server-core";
import { ServerRoot, renderToStringAsync } from "@portal/solidjs/server";
import Root from "~/app/root";
import { env } from "~/api/env";
import { createRepo } from "~/api/repo";
import { router } from "~/api/index";
import { EmbeddingsModel } from "@arena/cloud/llm";

const dbpool = new Pool({
  host: env.PORTAL_DATABASE_HOST,
  port: env.PORTAL_DATABASE_PORT,
  database: env.PORTAL_DATABASE_NAME,
  user: env.PORTAL_DATABASE_USER,
  password: env.PORTAL_DATABASE_PASSWORD,
});

const handler = chainMiddlewares<{ event: PageEvent }>(
  async ({ event }) => {
    const portalUser = event.request.headers.get("x-portal-user") || "null";
    let pool = dbpool.withDefaultAclChecker({
      user: JSON.parse(portalUser),
    });
    const repo = await createRepo({ pool });
    try {
      const result = await router.route(event.request, {
        env: process.env,
        context: {
          dbpool: pool,
          repo,
          llm: {
            embeddingsModel: new EmbeddingsModel({}),
          },
        },
      });

      // If the status code is 404 and the path isn't related to API or
      // registry, return undefined such that HTML renderer handles the
      // request
      if (result?.status == 404) {
        const url = new URL(event.request.url);
        if (
          !url.pathname.startsWith("/api") &&
          !url.pathname.startsWith("/registry")
        )
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
