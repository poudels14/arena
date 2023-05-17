CREATE TABLE apps (
  id VARCHAR(50) UNIQUE,
  name VARCHAR(100),
  description TEXT,
  workspace_id VARCHAR(50),
  config JSONB,
  owner_id VARCHAR(50),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  archived_at TIMESTAMPTZ DEFAULT NULL
);
