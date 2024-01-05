CREATE TABLE workspaces (
  id VARCHAR(50) UNIQUE,
  name VARCHAR(50) NOT NULL,
  config JSONB DEFAULT '{}',
  created_at TIMESTAMP,
  archived_at TIMESTAMP DEFAULT NULL
);
