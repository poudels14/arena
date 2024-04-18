import z from "zod";
import { createEnv } from "@t3-oss/env-core";
import { serve } from "@arena/runtime/server";
import { Client } from "@arena/runtime/postgres";
import { PostgresDatabaseMigrator } from "@portal/deploy/db/postgres";
// @ts-expect-error
import * as appServer from "@dqs/template/app";
import { migrateDatabase } from "@portal/deploy/db";
import { router as adminRouter } from "./admin";

const handler = appServer.default?.fetch;

if (appServer.migrations) {
  console.log("Running migrations");

  const env = createEnv({
    server: {
      PORTAL_DATABASE_HOST: z.string(),
      PORTAL_DATABASE_PORT: z.string().transform((val) => parseInt(val)),
      PORTAL_DATABASE_NAME: z.string(),
      PORTAL_DATABASE_USER: z.string(),
      PORTAL_DATABASE_PASSWORD: z.string(),
    },
    runtimeEnv: process.env,
    isServer: true,
  });

  const client = new Client({
    host: env.PORTAL_DATABASE_HOST,
    port: env.PORTAL_DATABASE_PORT,
    user: env.PORTAL_DATABASE_USER,
    password: env.PORTAL_DATABASE_PASSWORD,
    database: env.PORTAL_DATABASE_NAME,
  });

  const migrator = new PostgresDatabaseMigrator(client, appServer.migrations);
  await migrateDatabase(migrator, client, appServer.migrations);
  client.close();
  console.log("Migration completed!");
}
serve({
  async fetch(req) {
    const adminRes = await adminRouter.route(req);
    if (adminRes) {
      return adminRes;
    }
    if (handler) {
      const res = await handler(req);
      if (res instanceof Response) {
        return res;
      }
    }
    return new Response("Not found", {
      status: 404,
    });
  },
});
