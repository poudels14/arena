/**
 * Note(sagar): all the Arena modules used here should either be open-sourced
 * or it's alternative be available in NPM so that other developers can use
 * those modules when developing custom app templates.
 */
import {
  chainMiddlewares,
  createHandler,
  renderAsync,
} from "@arena/core/server";
import type { PageEvent } from "@arena/core/server";
import { ServerRoot } from "@arena/core/solid/server";
import { setupDatabase } from "@arena/sdk/db";
import { Flags, Client } from "@arena/runtime/sqlite";
import { createFileRouter } from "@arena/runtime/filerouter";
import { router } from "~/api";
import { databases } from "./server";
import { VectorDatabase } from "@arena/cloud/vectordb";
import Root from "~/root";

const dbs: any = {
  default: null,
  vectordb: null,
};

if (process.env.MODE == "development") {
  const mainDb = new Client({
    path: path.join("./data/db.sqlite"),
    flags: Flags.SQLITE_OPEN_CREATE | Flags.SQLITE_OPEN_READ_WRITE,
  });
  const vectordb = await VectorDatabase.open("./test-arena-cloud-vectordb");

  await setupDatabase(mainDb, mainDb, databases.default);
  await setupDatabase(mainDb, vectordb, databases.vectordb);

  dbs.default = mainDb;
  dbs.vectordb = vectordb;
}

const fileRouter = createFileRouter({
  env: {
    SSR: "false",
  },
  resolve: {
    preserveSymlink: true,
    conditions: ["solid", "browser"],
    alias: {
      "~": "./src",
    },
    dedupe: [
      "solid-js",
      "@solidjs/router",
      "@solidjs/meta",
      "@arena/core",
      "@arena/solid-store",
    ],
  },
});

const handler = chainMiddlewares<{ event: PageEvent }>(
  process.env.MODE == "development"
    ? async ({ event }) => {
        return fileRouter(event.request);
      }
    : null,
  async ({ event }) => {
    try {
      return await router.route(event.request, {
        env: event.env,
        context: {
          user: {
            id: "test-user",
          },
          dbs,
        },
      });
    } catch (e) {
      console.error(e);
    }
  },
  renderAsync(({ event }) => {
    return <ServerRoot event={event} Root={Root} />;
  })
);

const http = createHandler(async (event) => await handler({ event }));
export default {
  fetch: http.fetch,
};
