CREATE TABLE playoff_alliances(
  id INTEGER PRIMARY KEY NOT NULL,
  teams TEXT NOT NULL,  -- Vec<usize> of teams
  ready BOOLEAN NOT NULL DEFAULT 0
);