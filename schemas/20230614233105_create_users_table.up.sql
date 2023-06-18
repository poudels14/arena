CREATE TABLE users (
  id VARCHAR(50) UNIQUE,
  email VARCHAR(100),
  first_name VARCHAR(1000),
  last_name VARCHAR(1000),
  team_id VARCHAR(50) DEFAULT NULL,
  config JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  archived_at TIMESTAMPTZ DEFAULT NULL
);
