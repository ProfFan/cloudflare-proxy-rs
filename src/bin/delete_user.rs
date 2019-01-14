use cloudflare_proxy::db::{delete_user, establish_connection};
use std::io::stdin;

fn main() {
    let connection = establish_connection();

    println!("Input username:");
    let mut name = String::new();
    stdin().read_line(&mut name).unwrap();
    let name = &name[..(name.len() - 1)]; // Drop the newline character
    println!("\nReally delete {}? (y/n)\n", name);
    let mut key = String::new();
    stdin().read_line(&mut key).unwrap();

    if key.trim().to_lowercase() == "y" {
        let num_deleted = delete_user(&connection, name);
        println!("\nRemoved {} users with name {}", num_deleted, name);
    }
}
//fn main(){}
