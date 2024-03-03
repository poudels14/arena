import { PostgresDatabaseConfig } from "@portal/deploy/db";

const migrations: PostgresDatabaseConfig = {
  name: "main",
  type: "postgres",
  migrations: [
    {
      id: "create_chat_threads_table",
      async up(db) {
        await db.query(`CREATE TABLE chat_threads (
          -- thread id
          id          VARCHAR(100) NOT NULL,
          title       VARCHAR(500) NOT NULL,
          -- null if the thread isn't blocked
          blocked_by  VARCHAR(100),
          -- "user" | "app"
          created_by  VARCHAR(100),
          -- app id if created by app, else user id
          owned_id    VARCHAR(100),
          -- if not shared, only visible to the owner user
          shared      BOOL,
          metadata    JSONB NOT NULL,
          created_at   TIMESTAMP
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE chat_threads`);
      },
    },
    {
      id: "create_chat_messages_table",
      async up(db) {
        await db.query(`CREATE TABLE chat_messages (
          id          VARCHAR(100) NOT NULL,
          thread_id   VARCHAR(100),
          -- parent message id; this is set for AI response to a message
          parent_id   VARCHAR(100),
          role        VARCHAR(100) NOT NULL,
          user_id     VARCHAR(100),
          message     JSONB NOT NULL,
          -- in JSON format
          metadata    JSONB,
          created_at   TIMESTAMP
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE chat_messages`);
      },
    },
    {
      id: "create_task_executions_table",
      async up(db) {
        await db.query(`CREATE TABLE task_executions (
          id            VARCHAR(100) NOT NULL,
          -- id/name of the task
          task_id       VARCHAR(250) NOT NULL,
          thread_id     VARCHAR(100) NOT NULL,
          -- id of the message that triggered the task
          message_id    VARCHAR(100) NOT NULL,
          -- "STARTED" | "TERMINATED" | "COMPLETED" | "ERROR"
          status        VARCHAR(50) NOT NULL,
          metadata      JSONB NOT NULL,
          state         JSONB NOT NULL,
          started_at    TIMESTAMP NOT NULL
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE task_executions`);
      },
    },
    {
      id: "create_chat_artifacts_table",
      async up(db) {
        await db.query(`CREATE TABLE chat_artifacts (
          id VARCHAR(50) UNIQUE NOT NULL,
          name VARCHAR(250) NOT NULL,
          thread_id     VARCHAR(100) NOT NULL,
          message_id   VARCHAR(100) NOT NULL,
          size INTEGER NOT NULL,
          file FILE NOT NULL,
          metadata JSONB NOT NULL,
          created_at TIMESTAMP NOT NULL DEFAULT NOW(),
          archived_at TIMESTAMP DEFAULT NULL
        )`);
      },
      async down(db) {
        await db.query(`DROP TABLE chat_artifacts;`);
      },
    },
  ],
};

export default migrations;
