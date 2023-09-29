CREATE TABLE dqs_nodes (
  id VARCHAR(50) UNIQUE,
  host VARCHAR(1000),
  port INTEGER,
  status VARCHAR(25)
);
