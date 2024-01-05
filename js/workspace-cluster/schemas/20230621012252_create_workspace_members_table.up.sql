CREATE TABLE workspace_members (
  workspace_id VARCHAR(50) NOT NULL,
  user_id VARCHAR(50) NOT NULL,
  access VARCHAR(100) NOT NULL,
  added_at TIMESTAMP DEFAULT NOW(),
  archived_at TIMESTAMP DEFAULT NULL
);
