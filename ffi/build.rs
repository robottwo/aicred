extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("include")
        .join("genai_keyfinder.h");

    // Create the include directory if it doesn't exist
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create include directory");
    }

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_documentation(true)
        .with_include_guard("GENAI_KEYFINDER_H")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(output_file);
}
