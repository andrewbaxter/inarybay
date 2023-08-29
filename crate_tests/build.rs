use std::{
    env,
    path::PathBuf,
    str::FromStr,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let root = PathBuf::from_str(&env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    crate_tests_build::generate(root);
}
