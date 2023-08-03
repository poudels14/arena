export type AbstractDatabaseConfig<Config, MigrationClient> = {
  name: string;
  migrations: MigrationQuery<MigrationClient>[];
} & Config;

export type MigrationQuery<Client> = {
  up(db: Client): Promise<void>;
};

export type DbMigration = {
  id: number;
  /**
   * Name of the database
   */
  database: string;
  /**
   * Database type
   */
  type: string;
  hash: string;
};
