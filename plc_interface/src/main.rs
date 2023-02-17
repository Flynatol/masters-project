pub mod plc;

use plc::interface;

fn main() {
    println!("Hello, world!");

    let interface = interface::new();
}
