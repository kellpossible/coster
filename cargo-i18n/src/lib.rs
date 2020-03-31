//! `xtr`, and GNU Gettext CLI tools
//! `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
//! system path.

mod config;
mod error;
mod gettext;
mod util;

use std::path::Path;

use anyhow::{anyhow, Context, Result};

use walkdir::WalkDir;

fn cargo_refun_if_changed(path: &Path) -> anyhow::Result<()> {
    // println!(format!("cargo:rerun-if-changed={0}", path.to_str());
    Ok(())
}

/// Ensure that this script runs every time something within the gui
/// crate changes.
fn ensure_gui_watch_rerun() -> Result<()> {
    println!("cargo:rerun-if-changed=gui/Cargo.lock");
    for result in WalkDir::new("gui/") {
        match result {
            Ok(entry) => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
            Err(err) => return Err(anyhow!("error walking directory gui/: {}", err)),
        }
    }
    Ok(())
}
