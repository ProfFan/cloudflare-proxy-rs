#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use]
extern crate serde;

extern crate rocket;
extern crate rocket_contrib;

extern crate cloudflare;

pub mod db;
pub mod models;
pub mod schema;
