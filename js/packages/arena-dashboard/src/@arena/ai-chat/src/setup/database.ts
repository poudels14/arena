import {
  ArenaVectorDatabase,
  SqliteDatabaseConfig,
  SqliteDatabaseClient,
} from "@arena/sdk/db";

/**
 * Migrations for main database
 */
const main: SqliteDatabaseConfig = {
  name: "main",
  type: "sqlite",
  migrations: [
    {
      async up(mainDb: SqliteDatabaseClient) {
        await mainDb.query(`CREATE TABLE chat_history (
        id          TEXT NOT NULL,
        session_id  TEXT NOT NULL,
        thread_id   TEXT,
        -- parent message id; this is set for AI response to a message
        parent_id   TEXT,
        role        TEXT NOT NULL,
        user_id     TEXT,
        message     TEXT NOT NULL,
        timestamp   INTEGER
      )`);
      },
    },
    {
      async up(mainDb: SqliteDatabaseClient) {
        await mainDb.query(`CREATE TABLE uploads (
        id            TEXT NOT NULL,
        name          TEXT,
        content_hash  TEXT NOT NULL UNIQUE,
        content_type  TEXT NOT NULL,
        filename      TEXT NOT NULL,
        uploaded_at   INTEGER
      )`);
      },
    },
  ],
};

const vectordb: ArenaVectorDatabase.Config = {
  name: "vectordb",
  type: "arena-vectordb",
  migrations: [
    {
      async up(db: any) {
        await db.query(`CREATE TABLE uploads (dimension vector(384))`);
      },
    },
  ],
};

export { vectordb };
export default main;
