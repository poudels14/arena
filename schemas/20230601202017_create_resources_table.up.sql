CREATE TABLE resources (
  id VARCHAR(50) UNIQUE,
  workspace_id VARCHAR(50),
  name VARCHAR(255),
  description TEXT DEFAULT NULL,
  -- "@arena/sql/postgres", "env", "config",
  type VARCHAR(100),
  -- whether this resouce is a secret; value of secret resource is only visible
  -- to privileged code and not visible to user code
  secret BOOLEAN DEFAULT false,
  key VARCHAR(100) DEFAULT NULL,
  value JSONB,
  -- this is to support multiple "environments" like prod/staging
  context_id VARCHAR(50) DEFAULT NULL,
  created_by VARCHAR(50),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  archived_at TIMESTAMPTZ DEFAULT NULL
);

