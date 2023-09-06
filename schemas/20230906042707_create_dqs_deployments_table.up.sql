-- use this to keep track of deployed dqs servers instead
-- of using sth like etcd, or consul
CREATE TABLE dqs_deployments (
  id VARCHAR(50) UNIQUE NOT NULL,
  -- id of the cluster node that this server is deployed in
  node_id VARCHAR(50) NOT NULL,
  workspace_id VARCHAR(50) NOT NULL,
  app_id VARCHAR(50),
  app_template_id VARCHAR(50),
  started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_heartbeat_at TIMESTAMPTZ DEFAULT NULL,
  -- if this is set, the dqs server should be rebooted
  -- this is done to update things like env variables, etc
  reboot_triggered_at TIMESTAMPTZ DEFAULT NULL
);
