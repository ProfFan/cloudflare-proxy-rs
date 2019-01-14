CREATE TABLE user_site_privileges (
  id SERIAL PRIMARY KEY,
  user_id SERIAL REFERENCES users(id),
  site_id SERIAL REFERENCES sites(id),
  pattern VARCHAR,
  superuser BOOLEAN NOT NULL DEFAULT 'f'
)