use cloudflare_proxy::db::{create_site, establish_connection};
use std::io::stdin;

fn main() {
    let connection = establish_connection();

    println!("Input site name:");
    let mut name = String::new();
    stdin().read_line(&mut name).unwrap();
    let name = &name[..(name.len() - 1)]; // Drop the newline character
    println!("\nOk! Input the zone (domain name) for {}\n", name);
    let mut key = String::new();
    stdin().read_line(&mut key).unwrap();

    key.truncate(key.len() - 1);

    let post = create_site(&connection, name, &key);
    println!("\nCreated site {} with id {}", name, post.id);
}
