-- Your SQL goes here
CREATE TABLE teams (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name VARCHAR(250),
  affiliation VARCHAR(300),
  location VARCHAR(300),
  notes VARCHAR(500)
);