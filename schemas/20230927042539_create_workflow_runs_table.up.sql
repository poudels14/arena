CREATE TABLE workflow_runs (
  id VARCHAR(50) UNIQUE,
  workspace_id VARCHAR(50) NOT NULL,
  -- If triggered by an app, set this to be that app's id
  parent_app_id VARCHAR(50) DEFAULT NULL,
  -- { "plugin: { "id": "{id}", "version": "{version}", "workflow": "{name}" }
  template JSONB NOT NULL,
  config JSONB NOT NULL,
  state JSONB NOT NULL,
  -- "CREATED" | "IN-PROGRESS" | "ERRORED" | "ABORTED" | "COMPLETED"
  -- TODO: maybe track "WAITING-INPUT" (from user) as well?
  status VARCHAR(25) NOT NULL,
  triggered_by JSONB NOT NULL,
  triggered_at TIMESTAMPTZ DEFAULT NOW(),
  last_heartbeat_at TIMESTAMPTZ DEFAULT NULL,
  completed_at TIMESTAMPTZ DEFAULT NULL
);
