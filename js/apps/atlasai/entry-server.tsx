/**
 * Note(sagar): all the Arena modules used here should either be open-sourced
 * or it's alternative be available in NPM so that other developers can use
 * those modules when developing custom app templates.
 */
import { Pool } from "@arena/runtime/postgres";
import { chainMiddlewares, createHandler } from "@portal/server-core";
import type { PageEvent } from "@portal/server-core";
import { router } from "~/api/index";
import { env } from "./api/env";
import { createRepo } from "./api/repo";

const dbpool = new Pool({
  host: env.PORTAL_DATABASE_HOST,
  port: env.PORTAL_DATABASE_PORT,
  database: env.PORTAL_DATABASE_NAME,
  user: env.PORTAL_DATABASE_USER,
  password: env.PORTAL_DATABASE_PASSWORD,
});

const handler = chainMiddlewares<{ event: PageEvent }>(async ({ event }) => {
  const repo = await createRepo({ pool: dbpool });
  try {
    return await router.route(event.request, {
      env: event.env,
      context: {
        dbpool,
        repo,
        embeddingsModel: null!,
      },
    });
  } catch (e) {
    console.error(e);
  } finally {
    await repo.release();
  }
});

const http = createHandler(async (event) => await handler({ event }));
export default {
  fetch: http.fetch,
};

export { default as migrations } from "./migrations";
