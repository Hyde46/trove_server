-- Your SQL goes here
CREATE TABLE api_token (
  id SERIAL NOT NULL PRIMARY KEY,
  token TEXT NOT NULL,
  user_id SERIAL,
  revoked BOOLEAN NOT NULL DEFAULT FALSE,
  CONSTRAINT fk_user
      FOREIGN KEY(user_id) 
	  REFERENCES users(id)
);