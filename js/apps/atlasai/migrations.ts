import { PostgresDatabaseConfig } from "@portal/deploy/db";

const migrations: PostgresDatabaseConfig = {
  name: "main",
  type: "postgres",
  migrations: [
    {
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
      async down(db) {},
    },
  ],
};

export default migrations;
