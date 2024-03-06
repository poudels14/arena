import { Pool, Client } from "@arena/runtime/postgres";
import { drizzle } from "drizzle-orm/postgres-js";
import { createRepo as createUsersRepo } from "./users";
import { createRepo as createAclRepo } from "./acl";
import { createRepo as createWorkspacesRepo } from "./workspace";
import { createRepo as createSettingsRepo } from "./settings";
import { createRepo as createAppsRepo } from "./apps";
import { createRepo as createAppTemplatesRepo } from "./appTemplates";
import { createRepo as createDatabaseClusterRepo } from "./databaseClusters";
import { createRepo as createDatabaseRepo } from "./databases";
import { createRepo as createAppDeploymentsRepo } from "./appDeployments";

type Repo = {
  transaction(): Promise<
    Repo & {
      commit(): Promise<void>;
      rollback(): Promise<void>;
    }
  >;
  release(): Promise<void>;
  users: ReturnType<typeof createUsersRepo>;
  acl: ReturnType<typeof createAclRepo>;
  workspaces: ReturnType<typeof createWorkspacesRepo>;
  settings: ReturnType<typeof createSettingsRepo>;
  apps: ReturnType<typeof createAppsRepo>;
  appTemplates: ReturnType<typeof createAppTemplatesRepo>;
  appDeployments: ReturnType<typeof createAppDeploymentsRepo>;
  dbClusters: ReturnType<typeof createDatabaseClusterRepo>;
  databases: ReturnType<typeof createDatabaseRepo>;
};

const createRepo = async (options: { pool?: Pool; client?: Client }) => {
  const client = options.client ?? (await options.pool!.connect());
  let pg = drizzle(client);

  return {
    async transaction() {
      if (!options.pool) {
        throw new Error("Database pool must be passed when creating a repo");
      }
      const client = await options.pool!.connect();
      await client.query("BEGIN");
      const repo = await createRepo({ client });
      return Object.assign(repo, {
        async transaction() {
          throw new Error("Nested transaction not supported");
        },
        async commit() {
          await client.query("COMMIT");
        },
        async rollback() {
          await client.query("ROLLBACK");
        },
      });
    },
    async release() {
      // rollback just in case the client was release
      // in the middle of the transaction
      await client.query("ROLLBACK");

      // @ts-expect-error
      client?.release && (await client?.release());
    },
    users: createUsersRepo(pg),
    acl: createAclRepo(pg),
    workspaces: createWorkspacesRepo(pg),
    settings: createSettingsRepo(pg),
    apps: createAppsRepo(client),
    appTemplates: createAppTemplatesRepo(pg),
    appDeployments: createAppDeploymentsRepo(pg),
    dbClusters: createDatabaseClusterRepo(pg),
    databases: createDatabaseRepo(pg),
  };
};

export { createRepo };
export type { Repo };
