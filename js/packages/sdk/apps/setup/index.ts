import { SqliteMigrator } from "./migration";
import type { DatabaseClient, MigrationQuery } from "../db";

const setup = async (options: {
  client: DatabaseClient;
  migrations: MigrationQuery[];
}) => {
  console.log("[setup] Running database migrations...");
  await new SqliteMigrator(options.client).migrate(options.migrations);
};

export { setup };
