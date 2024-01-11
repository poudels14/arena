import { PostgresDatabaseConfig } from "@portal/deploy/db";

const migrations: PostgresDatabaseConfig = {
  name: "main",
  type: "postgres",
  migrations: [
    {
      async up(db) {
        await db.query(`CREATE TABLE users (
          id VARCHAR(50) UNIQUE,
          email VARCHAR(100) UNIQUE,
          first_name VARCHAR(1000),
          last_name VARCHAR(1000),
          team_id VARCHAR(50) DEFAULT NULL,
          config JSONB,
          created_at TIMESTAMP DEFAULT NOW(),
          archived_at TIMESTAMP DEFAULT NULL
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE users;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE workspaces (
          id VARCHAR(50) UNIQUE,
          name VARCHAR(50) NOT NULL,
          config JSONB DEFAULT '{}',
          created_at TIMESTAMP,
          archived_at TIMESTAMP DEFAULT NULL
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE workspaces;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE workspace_members (
          workspace_id VARCHAR(50) NOT NULL,
          user_id VARCHAR(50) NOT NULL,
          access VARCHAR(100) NOT NULL,
          added_at TIMESTAMP DEFAULT NOW(),
          archived_at TIMESTAMP DEFAULT NULL
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE workspace_members;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE database_clusters (
          id VARCHAR(50) UNIQUE NOT NULL,
          host VARCHAR(250) NOT NULL,
          port INTEGER NOT NULL,
          capacity INTEGER NOT NULL,
          usage INTEGER NOT NULL,
          credentials JSONB
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE database_clusters;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE databases (
          id VARCHAR(50) UNIQUE NOT NULL,
          workspace_id VARCHAR(50) NOT NULL,
          app_id VARCHAR(50),
          credentials JSONB,
          -- cluster_id is NULL if the database if offline
          cluster_id VARCHAR(50)
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE databases;`);
      },
    },
  ],
};

export default migrations;
