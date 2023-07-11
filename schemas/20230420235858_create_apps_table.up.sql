CREATE TABLE apps (
  id VARCHAR(50) UNIQUE,
  name VARCHAR(100),
  description TEXT,
  workspace_id VARCHAR(50),
  template JSONB DEFAULT NULL,
  config JSONB,
  created_by VARCHAR(50),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  archived_at TIMESTAMPTZ DEFAULT NULL
);
