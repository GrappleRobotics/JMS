-- Your SQL goes here

CREATE TABLE matches (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  match_type TEXT NOT NULL,
  set_number INTEGER NOT NULL,
  match_number INTEGER NOT NULL,
  red_teams TEXT NOT NULL,  -- JSON Array
  blue_teams TEXT NOT NULL  -- JSON Array
);