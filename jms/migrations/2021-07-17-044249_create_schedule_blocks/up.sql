CREATE TABLE schedule_blocks(
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL DEFAULT "Untitled Block",
  start_time BIGINT NOT NULL,
  end_time BIGINT NOT NULL,
  cycle_time BIGINT NOT NULL DEFAULT 780000,  -- 13 minutes default
  quals BOOLEAN NOT NULL DEFAULT 0
)