use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use crate::models::{NewUser, User};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_user<'a>(conn: &PgConnection, name: &'a str, key: &'a str) -> User {
    use crate::schema::users;

    let new_user = NewUser { name, key };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn delete_user(conn: &PgConnection, _name: &str) -> usize {
    use crate::schema::users;

    let usr = users::table.filter(users::name.eq(_name));

    diesel::delete(usr)
        .execute(conn)
        .expect("Error deleting user")
}
