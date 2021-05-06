-- Your SQL goes here
CREATE TABLE teams (
  id INTEGER PRIMARY KEY NOT NULL,
  name VARCHAR(250) NOT NULL,
  affiliation VARCHAR(300),
  location VARCHAR(300),
  notes VARCHAR(500)
);