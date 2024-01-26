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
    {
      async up(db) {
        await db.query(`CREATE TABLE apps (
          id VARCHAR(50) UNIQUE,
          name VARCHAR(100),
          slug VARCHAR(100),
          description TEXT,
          workspace_id VARCHAR(50),
          template JSONB DEFAULT NULL,
          config JSONB,
          created_by VARCHAR(50),
          created_at TIMESTAMPTZ DEFAULT NOW(),
          updated_at TIMESTAMPTZ DEFAULT NOW(),
          archived_at TIMESTAMPTZ DEFAULT NULL
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE apps;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE UNIQUE INDEX apps_workspace_id_slug ON apps (
          workspace_id,
          slug
        );`);
      },
      async down(db) {
        await db.query(`DROP INDEX apps_workspace_id_slug;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE app_clusters (
          id VARCHAR(50) UNIQUE,
          host VARCHAR(1000),
          port INTEGER,
          status VARCHAR(25),
          started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE app_clusters;`);
      },
    },
    {
      async up(db) {
        await db.query(`
        -- use this to keep track of deployed apps instead
        -- of using sth like etcd, or consul
        CREATE TABLE app_deployments (
          id VARCHAR(50) UNIQUE NOT NULL,
          -- id of the cluster node that this server is deployed in
          node_id VARCHAR(50) NOT NULL,
          workspace_id VARCHAR(50) NOT NULL,
          app_id VARCHAR(50),
          app_template_id VARCHAR(50),
          started_at TIMESTAMP NOT NULL DEFAULT NOW(),
          last_heartbeat_at TIMESTAMP DEFAULT NULL,
          -- if this is set, the dqs server should be rebooted
          -- this is done to update things like env variables, etc
          reboot_triggered_at TIMESTAMP DEFAULT NULL
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE app_deployments;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE environment_variables (
          id VARCHAR(50) UNIQUE,
          workspace_id VARCHAR(50) DEFAULT NULL,
          name VARCHAR(255) NOT NULL,
          description TEXT DEFAULT NULL,
          -- Only set if this env variable is provided by the app template author
          -- This variable is accessible only from the app template running in
          -- Arena cloud.
          -- If the app template allows env variable to be configurable when
          -- "installing" the app by an user, the app_id and "app_template_id"
          -- will both be set and that will override the env variable with same "key"
          -- having same "app_template_id".
          app_template_id VARCHAR(50) DEFAULT NULL,
          app_id VARCHAR(50) DEFAULT NULL,
          key VARCHAR(100) NOT NULL,
          value TEXT NOT NULL,
          created_by VARCHAR(50) DEFAULT NULL,
          created_at TIMESTAMP DEFAULT NOW(),
          updated_at TIMESTAMP DEFAULT NOW(),
          archived_at TIMESTAMP DEFAULT NULL
        );`);
      },
      async down(db) {
        await db.query(`DROP TABLE environment_variables;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE acls (
            id VARCHAR(50) UNIQUE,
            workspace_id VARCHAR(50) NOT NULL,
            user_id VARCHAR(50) NOT NULL,
            access VARCHAR(100) NOT NULL,
            app_id VARCHAR(50) DEFAULT NULL,
            -- If an app has multiple paths, different paths could have different
            -- access control
            path VARCHAR(50) DEFAULT NULL,
            resource_id VARCHAR(50) DEFAULT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            archived_at TIMESTAMPTZ DEFAULT NULL
          );`);
      },
      async down(db) {
        await db.query(`DROP TABLE acls;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE app_templates (
            -- unique app template id
            id VARCHAR(50) UNIQUE,
            name VARCHAR(1000) NOT NULL,
            description TEXT,
            default_version VARCHAR(50),
            owner_id VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            archived_at TIMESTAMPTZ DEFAULT NULL
          );`);
      },
      async down(db) {
        await db.query(`DROP TABLE app_templates;`);
      },
    },
  ],
};

export default migrations;
