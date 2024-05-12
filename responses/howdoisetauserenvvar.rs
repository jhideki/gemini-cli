use std::env;

fn main() {
    env::set_var("MY_VAR", "my_value");
    println!("MY_VAR is now set to {}", env::var("MY_VAR").unwrap());
}
