# `cloudflare-proxy`

Proxy for Cloudflare API written in Rust. This is mainly designed to solve the problem that Cloudflare only has one API key for all sites belonging to an account.

This program provides a Certbot DNS challenge provider [`certbot-dns-cfproxy`](https://github.com/ProfFan/certbot-dns-cfproxy) that you can install via Pypi for easy tiered management of SSL certs for your personal servers while not leaking your CF API key everywhere on different cloud server providers.

# Usage

First install PostgreSQL, create the user and database, change the `.env` file (follow the http://diesel.rs/ tutorial):

```
DATABASE_URL=postgres://user_pass@localhost/DB_NAME
```

Then create a user, a site and give the user privilege:

```
diesel setup

> cargo run --bin new_user
Input username:
cargo

Ok! Input the API Key for cargo

super_secret_key

Created user cargo with id 2

> cargo run --bin new_site
Input site name:
G00gle

Ok! Input the zone (domain name) for G00gle

google.com

Created site G00gle with id 2

> cargo run --bin new_priv
Input user:
cargo

Ok! Input the zone (domain name):

google.com
Input the regex pattern:
.*\.google\.com
Make him superuser for cargo: (y/N)
n
```

And run the web app with you cloudflare credentials:

```
ROCKET_CFUSER=<CF_EMAIL> ROCKET_CFKEY=<CF_KEY> cargo run
```

Now you can call the API with:

```
curl --verbose  --header "Content-Type: application/json" \
  --data '{"user":"username","key":"fdsfdafsas","zone":"google.com","rec":"vu1.google.com","rectype":"A", "value":"10.2.22.2"}' \
  http://localhost:8000/update
```

# ACME (Let's Encrypt)

Use with [`certbot-dns-cfproxy`](https://github.com/ProfFan/certbot-dns-cfproxy)

# LICENSE

MIT or Apache
