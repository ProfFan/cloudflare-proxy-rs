CREATE TABLE user_site_privileges (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users(id) NOT NULL,
  site_id INTEGER REFERENCES sites(id) NOT NULL,
  pattern VARCHAR NOT NULL,
  superuser BOOLEAN NOT NULL DEFAULT 'f'
)