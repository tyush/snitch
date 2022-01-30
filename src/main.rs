use std::env;

use snitch::scan;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    match env::args().nth(1) {
        Some(p) => scan(p),
        None => scan(".".to_owned())
    }
}
