import process from "process";
import { migrateDatabase } from "@portal/deploy/db";
import { Client } from "@arena/runtime/postgres";
import { PostgresDatabaseMigrator } from "@portal/deploy/db/postgres";
// TODO: only incude migrations necessary in order to avoid exposing
// migrations for cloud
import migrations from "@portal/workspace-cluster/migrations";

async function migrate(databaseUrl: string) {
  console.log("Starting database migration...");
  const client = new Client(databaseUrl);
  const migrator = new PostgresDatabaseMigrator(client, migrations);
  await migrateDatabase(migrator, client, migrations);
  console.log("Migration completed!");
}

migrate(process.env.DATABASE_URL!);
