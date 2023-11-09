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
        await mainDb.query(`CREATE TABLE chat_channels (
          -- channel id
          id          TEXT NOT NULL,
          name        TEXT,
          metadata    TEXT -- in JSON format
        );`);
      },
    },
    {
      async up(mainDb: SqliteDatabaseClient) {
        await mainDb.query(`
          INSERT INTO chat_channels (id, name, metadata)
          VALUES ('default', 'Default', '{"enableAI": true}');`);
      },
    },
    {
      async up(mainDb: SqliteDatabaseClient) {
        await mainDb.query(`CREATE TABLE chat_threads (
          -- thread id
          id          TEXT NOT NULL,
          channel_id  TEXT NOT NULL,
          title       TEXT NOT NULL,
          -- null if the thread isn't blocked
          blocked_by  TEXT,
          metadata    TEXT NOT NULL, -- in JSON format
          timestamp   INTEGER
        );`);
      },
    },
    {
      async up(mainDb: SqliteDatabaseClient) {
        await mainDb.query(`CREATE TABLE chat_messages (
          id          TEXT NOT NULL,
          channel_id  TEXT NOT NULL,
          thread_id   TEXT,
          -- parent message id; this is set for AI response to a message
          parent_id   TEXT,
          role        TEXT NOT NULL,
          user_id     TEXT,
          message     TEXT NOT NULL,
          -- in JSON format
          metadata    TEXT,
          timestamp   INTEGER
        );`);
      },
    },
    {
      async up(mainDb: SqliteDatabaseClient) {
        /**
         * Note(sagar): Arena platform will keep track of actively running
         * workflow and data related to that. But once the workflow is
         * completed/cancelled, it will clear all the data. So, workflow
         * run data that needs to be persisted should be stored here.
         */
        await mainDb.query(`CREATE TABLE workflow_runs (
          id            TEXT NOT NULL,
          channel_id    TEXT NOT NULL,
          thread_id     TEXT NOT NULL,
          -- JSON: { "id": "{id}", "version": "{version}" }
          plugin        TEXT NOT NULL,
          workflow_slug TEXT NOT NULL,
          -- config JSONB NOT NULL,
          -- state JSONB NOT NULL,
          -- status: | "CREATED" | "RUNNING" | "WAITING-INPUT" (from user) |
          --         | "PAUSED"  | "ERRORED" | "ABORTED" | "COMPLETED" |
          status        TEXT NOT NULL,
          triggered_at  INTEGER NOT NULL,
          completed_at  INTEGER
        );`);
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
        );`);
      },
    },
    {
      async up(mainDb: SqliteDatabaseClient) {
        await mainDb.query(`CREATE TABLE installed_plugins (
          id            TEXT NOT NULL,
          name          TEXT NOT NULL,
          description   TEXT,
          version       TEXT NOT NULL,
          installed_at  INTEGER NOT NULL
        );`);
      },
    },
    // {
    //   async up(mainDb: SqliteDatabaseClient) {
    //     await mainDb.query(`CREATE TABLE workflow-runs (
    //     id               TEXT NOT NULL,
    //     workflow_id      TEXT NOT NULL,
    //     workflow_version TEXT NOT NULL,
    //     started_at     INTEGER NOT NULL
    //   )`);
    //   },
    // },
    // {
    //   async up(mainDb: SqliteDatabaseClient) {
    // TODO: idk whether to store the "logs" in a separate table
    //     await mainDb.query(`CREATE TABLE workflow-run-logs (
    //     id               TEXT NOT NULL,
    //     workflow_id      TEXT NOT NULL,
    //     workflow_version TEXT NOT NULL,
    //     started_at     INTEGER NOT NULL
    //   )`);
    //   },
    // },
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
    {
      async up(db: any) {
        await db.query(`CREATE TABLE plugin_functions (dimension vector(384))`);
      },
    },
  ],
};

export { vectordb };
export default main;
