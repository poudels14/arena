CREATE TABLE env_variables (
  id VARCHAR(50) UNIQUE,
  -- NULL if the env variable isn't workspace specific, for example if it's
  -- provided by an app template
  workspace_id VARCHAR(50) DEFAULT NULL,
  -- Only set if this env variable is provided by the app template author
  -- This variable is accessible only from the app template running in
  -- Arena cloud.
  -- If the app template allows env variable to be configurable when
  -- "installing" the app by an user, the app_id and `app_template_id`
  -- will both be set and that will override the env variable with same `key`
  -- having same `app_template_id`.
  app_template_id VARCHAR(50) DEFAULT NULL,

  app_id VARCHAR(50) DEFAULT NULL,

  -- env variable key
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
