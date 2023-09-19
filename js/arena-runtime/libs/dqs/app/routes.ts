import { createRouter, procedure } from "@arena/runtime/server";
import {
  DatabaseConfig,
  DatabaseClients,
  setupDatabase,
  SqliteDatabaseClient,
} from "@arena/sdk/db";
// @ts-expect-error
import { databases } from "@dqs/template/app";

const p = procedure<{ dbs: DatabaseClients<{}> }>();
const router = createRouter({
  // Routes with `/_admin` prefix are only accessible by Arena cloud and
  // aren't exposed to the public
  routes: {
    "/_admin/healthy": p.query(async ({ ctx }) => {
      return "Ok";
    }),
    "/_admin/setup": p.mutate(async ({ ctx }) => {
      try {
        await setupDatabases(ctx.dbs["default"], ctx.dbs);
      } catch (e: any) {
        return { error: e.message };
      }

      return { success: "true" };
    }),
    // TODO(sagar): add bunch of endpoints under `/_metadata` that provides
    // information about the app like, what permissions this app needs
    // when installing, what resources are needed to be installed in the
    // workspace that this app will need to access, etc
    "/_metadata/permissions": p.query(async () => {}),
    // TODO(sagar): this will return the schema of the routes this app has
    // which can be used by LLM models.
    // TODO: maybe this endpoint should be under `/_admin`?
    "/_metadata/api/schemas": p.query(async () => {}),
  },
});

const setupDatabases = async (
  auditClient: SqliteDatabaseClient,
  dbs: DatabaseClients<{}>
) => {
  for (const [dbName, db] of Object.entries(
    databases as Record<string, DatabaseConfig>
  )) {
    await setupDatabase(auditClient, dbs[dbName], db);
  }
};

export { router };
