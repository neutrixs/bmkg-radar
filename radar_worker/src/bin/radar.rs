use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args = args.drain(1..).collect();

    let place = args.join(" ");
    println!("{}", place);
}