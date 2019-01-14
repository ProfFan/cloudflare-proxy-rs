# `cloudflare-proxy`

Proxy for Cloudflare API written in Rust.

# Usage

First change the `.env` file:

```
DATABASE_URL=postgres://user_pass@localhost/DB_NAME
```

Then create a user:

```
cargo run --bin new_user
```

And run the web app with:

```
ROCKET_CFUSER=<CF_EMAIL> ROCKET_CFKEY=<CF_KEY> cargo run
```
