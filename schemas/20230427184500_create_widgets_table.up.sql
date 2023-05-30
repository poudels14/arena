CREATE TABLE widgets (
  id VARCHAR(50) UNIQUE,
  name VARCHAR(255) DEFAULT NULL,
  slug VARCHAR(255) DEFAULT NULL,
  description TEXT DEFAULT NULL,
  app_id VARCHAR(50),
  template_id VARCHAR(50),
  parent_id VARCHAR(50) DEFAULT NULL,
  config JSONB,
  created_by VARCHAR(50),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  archived_at TIMESTAMPTZ DEFAULT NULL
);
