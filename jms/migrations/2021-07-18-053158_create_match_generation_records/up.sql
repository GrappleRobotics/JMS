-- Your SQL goes here
CREATE TABLE match_generation_records(
  -- id INTEGER PRIMARY KEY NOT NULL,
  match_type TEXT PRIMARY KEY NOT NULL,
  data TEXT    -- Depends on match_type
  -- team_balance NUMBER,
  -- station_balance NUMBER,
  -- cooccurrence TEXT, -- Vec<Vec<usize>>
  -- station_dist TEXT  -- Vec<Vec<usize>>
);