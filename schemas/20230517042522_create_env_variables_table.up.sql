CREATE TABLE env_variables (
  id VARCHAR(50) UNIQUE,
  workspace_id VARCHAR(50),
  key VARCHAR(100),
  value JSONB,
  -- "config" | "secret", etc
  type VARCHAR(100),
  -- this is to support multiple "environments" like prod/staging
  context_id VARCHAR(50) DEFAULT NULL,
  created_by VARCHAR(50),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  last_modified TIMESTAMPTZ DEFAULT NOW()
);
