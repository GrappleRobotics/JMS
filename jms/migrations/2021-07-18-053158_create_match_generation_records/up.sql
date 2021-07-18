-- Your SQL goes here
CREATE TABLE match_generation_records(
  id INTEGER PRIMARY KEY NOT NULL,
  team_balance NUMBER NOT NULL,
  station_balance NUMBER NOT NULL,
  cooccurrence TEXT NOT NULL, -- Vec<Vec<usize>>
  station_dist TEXT NOT NULL  -- Vec<Vec<usize>>
);