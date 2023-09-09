CREATE TABLE resource_types (
  id VARCHAR(100) UNIQUE,
  name VARCHAR(255),
  description TEXT DEFAULT NULL,
  config JSONB DEFAULT '{}',
  is_secret BOOL DEFAULT false,
  archived_at TIMESTAMPTZ DEFAULT NULL
);

INSERT INTO resource_types (id, name)
  VALUES
    ('@arena/sql/postgres', 'Postgres Database'),
    ('@arena/env', 'Environment variable');