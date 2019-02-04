# `cloudflare-proxy`

Proxy for Cloudflare API written in Rust.

# Usage

First change the `.env` file:

```
DATABASE_URL=postgres://user_pass@localhost/DB_NAME
```

Then create a user, a site and give the user privilege:

```
diesel setup

cargo run --bin new_user

cargo run --bin new_site

cargo run --bin new_priv
```

And run the web app with:

```
ROCKET_CFUSER=<CF_EMAIL> ROCKET_CFKEY=<CF_KEY> cargo run
```

Now you can call the API with:

```
curl --verbose  --header "Content-Type: application/json" \
  --data '{"user":"username","key":"fdsfdafsas","zone":"example.net","rec":"vu1.example.net","rectype":"A", "value":"10.2.22.2"}' \
  http://localhost:8000/update
```

# ACME (Let's Encrypt)

Use with [`certbot-dns-cfproxy`](https://github.com/ProfFan/certbot-dns-cfproxy)

# LICENSE

MIT or Apache
