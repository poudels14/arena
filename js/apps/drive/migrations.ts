import { PostgresDatabaseConfig } from "@portal/deploy/db";

const migrations: PostgresDatabaseConfig = {
  name: "main",
  type: "postgres",
  migrations: [
    {
      async up(db) {
        await db.query(`CREATE TABLE files (
          id VARCHAR(50) UNIQUE NOT NULL,
          name VARCHAR(250) NOT NULL,
          description TEXT,
          -- parent id is either parent directory id or parent file id
          -- if this file was derived, set id of the original file
          -- derived files are used to stored extracted text from
          -- pdf, audio, etc
          parent_id VARCHAR(50),
          is_directory BOOL,
          size INTEGER NOT NULL DEFAULT 0,
          file FILE,
          content_type VARCHAR(100) DEFAULT NULL,
          metadata JSONB,
          -- id of the user who created the file/directory
          created_by VARCHAR(50),
          created_at TIMESTAMP NOT NULL DEFAULT NOW(),
          updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
          archived_at TIMESTAMP DEFAULT NULL
        )`);
      },
      async down(db) {
        await db.query(`DROP TABLE files;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE INDEX files_parent_id ON files(
          parent_id
        )`);
      },
      async down(db) {
        await db.query(`DROP INDEX files_parent_id;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE TABLE file_embeddings (
          id VARCHAR(50) UNIQUE NOT NULL,
          -- "file" | "message" | etc
          -- source_type VARCHAR(50) NOT NULL,
          -- file id if source_type == "file", ...
          file_id VARCHAR(50),
          directory_id VARCHAR(50),
          metadata JSONB,
          embeddings VECTOR(384) NOT NULL,
          created_at TIMESTAMP NOT NULL DEFAULT NOW
        )`);
      },
      async down(db) {
        await db.query(`DROP TABLE file_embeddings;`);
      },
    },
    {
      async up(db) {
        await db.query(`CREATE INDEX embeddings_file_id_key ON file_embeddings(
          file_id
        )`);
      },
      async down(db) {
        await db.query(`DROP INDEX embeddings_file_id_key;`);
      },
    },
  ],
};

export default migrations;
