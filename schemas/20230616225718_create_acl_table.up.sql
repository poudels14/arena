CREATE TABLE acl (
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
);
