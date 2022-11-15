CREATE TABLE api_token (
  id SERIAL NOT NULL PRIMARY KEY,
  token TEXT NOT NULL,
  user_id_fk SERIAL,
  revoked BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL,
  CONSTRAINT fk_user
      FOREIGN KEY(user_id_fk) 
	  REFERENCES users(id)
);