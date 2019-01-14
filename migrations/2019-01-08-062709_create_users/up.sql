CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  key VARCHAR NOT NULL,
  disabled BOOLEAN NOT NULL DEFAULT 'f'
)
