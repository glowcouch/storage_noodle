//! Build script that copies README to docs.
use std::{env, error::Error, fs, path::PathBuf};

/// Crate name.
const CRATE_NAME: &str = "storage_noodle_traits";

/// Path to Readme.
const README_PATH: &str = "../README.md";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={README_PATH}");
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("README-rustdocified.md"),
        readme_rustdocifier::rustdocify(
            &fs::read_to_string(README_PATH)?,
            &env::var("CARGO_PKG_NAME")?,
            Some(&env::var("CARGO_PKG_VERSION")?),
            Some(CRATE_NAME),
        )?,
    )?;
    Ok(())
}
