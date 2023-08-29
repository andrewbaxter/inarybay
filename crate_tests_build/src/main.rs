use std::env::current_dir;
use crate_tests_build::generate;

pub fn main() {
    generate(current_dir().unwrap());
}
