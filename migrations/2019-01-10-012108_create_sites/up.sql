CREATE TABLE sites (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  zone VARCHAR NOT NULL UNIQUE,
  disabled BOOLEAN NOT NULL DEFAULT 'f'
)