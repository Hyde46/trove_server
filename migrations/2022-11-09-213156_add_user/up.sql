CREATE TABLE users (
  id SERIAL NOT NULL PRIMARY KEY,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  email TEXT NOT NULL,
  pw_hash TEXT NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMP NOT NULL
);