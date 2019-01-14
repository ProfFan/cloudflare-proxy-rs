use cloudflare_proxy::db::{create_user, establish_connection};
use std::io::stdin;

fn main() {
    let connection = establish_connection();

    println!("Input username:");
    let mut name = String::new();
    stdin().read_line(&mut name).unwrap();
    let name = &name[..(name.len() - 1)]; // Drop the newline character
    println!("\nOk! Input the API Key for {}\n", name);
    let mut key = String::new();
    stdin().read_line(&mut key).unwrap();

    key.truncate(key.len() - 1);

    let post = create_user(&connection, name, &key);
    println!("\nCreated user {} with id {}", name, post.id);
}
//fn main(){}
