use cloudflare_proxy::db::establish_connection;
use cloudflare_proxy::models::{NewUserSitePrivilege, User};
use diesel::prelude::*;
use std::io::stdin;

fn main() {
    let mut connection = establish_connection();

    println!("Input user:");
    let mut name = String::new();
    stdin().read_line(&mut name).unwrap();
    let name = &name[..(name.len() - 1)]; // Drop the newline character

    use cloudflare_proxy::schema::users;

    let usr = users::table
        .filter(users::name.eq(name))
        .load::<User>(&mut connection)
        .expect("Error loading users");

    if usr.len() != 1 {
        eprintln!("User not found!");
        return;
    }

    println!("\nOk! Input the zone (domain name):\n");
    let mut zone_name = String::new();
    stdin().read_line(&mut zone_name).unwrap();

    zone_name.truncate(zone_name.len() - 1);

    use cloudflare_proxy::schema::sites;

    let site = sites::table
        .filter(sites::zone.eq(zone_name))
        .load::<User>(&mut connection)
        .expect("Error loading sites!");

    if site.len() != 1 {
        eprintln!("Zone not found!");
        return;
    }

    println!("Input the regex pattern:");
    let mut pattern = String::new();
    stdin().read_line(&mut pattern).unwrap();

    pattern.truncate(pattern.len() - 1);

    println!("Make him superuser for {}: (y/N)", name);
    let mut superuser = false;
    let mut _superuser = String::new();
    stdin().read_line(&mut _superuser).unwrap();

    _superuser.truncate(_superuser.len() - 1);

    if _superuser.to_uppercase() == "Y" {
        superuser = true;
    }

    use cloudflare_proxy::schema::user_site_privileges;

    let new_priv = NewUserSitePrivilege {
        user_id: usr[0].id,
        site_id: site[0].id,
        pattern: &pattern,
        superuser,
    };

    diesel::insert_into(user_site_privileges::table)
        .values(&new_priv)
        .execute(&mut connection)
        .expect("Error saving new privilege");
}
