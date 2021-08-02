-- Your SQL goes here

CREATE TABLE matches (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  start_time BIGINT,
  match_type TEXT NOT NULL,
  set_number INTEGER NOT NULL,
  match_number INTEGER NOT NULL,
  blue_teams TEXT NOT NULL,  -- JSON Array
  red_teams TEXT NOT NULL,  -- JSON Array
  played BOOLEAN NOT NULL,
  score TEXT,
  winner TEXT
);