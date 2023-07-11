export type DatabaseClient = {
  query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
  transaction<T>(closure: () => T | Promise<T>): Promise<void>;
};

export type MigrationQuery = {
  up(db: Pick<DatabaseClient, "query">): Promise<void>;
};

export type DbMigration = {
  id: number;
  hash: string;
};
