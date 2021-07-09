-- Your SQL goes here
CREATE TYPE match_type AS ENUM ('Test', 'Qualification', 'Quarterfinal', 'Semifinal', 'Final');

CREATE TABLE matches (
  id SERIAL PRIMARY KEY,
  match_type match_type NOT NULL,
  set_number INTEGER NOT NULL,
  match_number INTEGER NOT NULL,
  red_teams INTEGER[] NOT NULL,
  blue_teams INTEGER[] NOT NULL
);