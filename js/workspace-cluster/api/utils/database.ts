import { Client } from "@arena/runtime/postgres";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { Repo } from "../repo";

const addDatabase = async (
  repo: Repo,
  database: {
    id: string;
    workspaceId: string;
    appId?: string;
    user: string;
  }
) => {
  const clusters = await repo.dbClusters.list({});
  const cluster = clusters.find((cluster) => {
    return cluster.capacity - cluster.usage > 0;
  });

  if (!cluster) {
    throw new Error("No database capacity left");
  }

  const credentials = cluster.credentials;
  const adminClient = new Client(
    `postgresql://${credentials.adminUser}:${credentials.adminPassword}@${cluster.host}:${cluster.port}/postgres`
  );

  const dbName = encodeURI(database.id);
  try {
    await adminClient.query(`CREATE database ${dbName}`);

    const password = uniqueId(29);
    await adminClient.query(
      `EXECUTE arena_set_catalog_user_credential('${dbName}', 'app', '${password}')`
    );

    const newDatabase = await repo.databases.add({
      id: dbName,
      workspaceId: database.workspaceId,
      appId: database.appId || null,
      clusterId: cluster.id,
      credentials: {
        user: database.user,
        password,
      },
    });
    return newDatabase;
  } finally {
    adminClient.close();
  }
};

export { addDatabase };
