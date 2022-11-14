CREATE TABLE trove (
  id SERIAL NOT NULL PRIMARY KEY,
  trove_text TEXT NOT NULL,
  user_id_fk SERIAL,
  created_at TIMESTAMP NOT NULL,
  CONSTRAINT fk_user
      FOREIGN KEY(user_id_fk) 
	  REFERENCES users(id)
);