use std::env;
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, ensure, Context, Result};
use ignore::Walk;
use serde_derive::Deserialize;
use toml;
use tr::tr;

fn main() {
    //TODO: fix this disabled building of the wasm frontend
    // build_wasm_frontend().expect("error while building the wasm front-end");
}

#[derive(Deserialize)]
struct BuildConfig {}

fn read_toml_config() -> Result<BuildConfig> {
    let toml_path = Path::new("build.toml");
    let toml_str = read_to_string(toml_path).context("trouble reading Cargo.toml")?;
    let config: BuildConfig =
        toml::from_str(toml_str.as_ref()).context("trouble parsing Cargo.toml")?;
    println!("cargo:rerun-if-changed=Build.toml");
    Ok(config)
}

/// Build the wasm component used for the front end of the website.
/// Requires `wasm-pack` CLI, `xtr`, and GNU Gettext CLI tools
/// `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
/// system path.
///
/// Runs the command `wasm-pack build --target web --out-dir
/// ../public/js/gui`
fn build_wasm_frontend() -> Result<()> {
    let config = read_toml_config()?;
    println!("cargo:rerun-if-changed=Build.toml");
    ensure_gui_watch_rerun()?;
    build_wasm()?;

    // enable panic here for debugging due to a stupid feature where
    // stdout from this module isn't even included in cargo build -vv.
    // panic!("debugging");
    Ok(())
}

/// Ensure that this script runs every time something within the gui
/// crate changes.
fn ensure_gui_watch_rerun() -> Result<()> {
    println!("cargo:rerun-if-changed=gui/Cargo.lock");
    for result in Walk::new("gui/") {
        match result {
            Ok(entry) => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
            Err(err) => return Err(anyhow!("error walking directory gui/: {}", err)),
        }
    }
    Ok(())
}

fn run_command_and_check_success(command_name: &str, mut command: Command) -> Result<()> {
    let output = command
        .spawn()
        .with_context(|| tr!("the {0} command was unable to start", command_name))?
        .wait_with_output()
        .with_context(|| {
            tr!(
                "the {0} command had a problem waiting for output",
                command_name
            )
        })?;

    ensure!(
        output.status.success(),
        tr!(
            "the {0} command reported that it was unsuccessful",
            command_name
        )
    );
    Ok(())
}

fn build_wasm() -> Result<()> {
    let profile: String = env::var("PROFILE").unwrap();

    let mut wasm_pack = Command::new("wasm-pack");
    wasm_pack.current_dir("./gui");
    wasm_pack.args(&["build", "--target", "web", "--out-dir", "../public/js/gui"]);

    if profile == "debug" {
        wasm_pack.arg("--dev");
    }

    run_command_and_check_success("wasm-pack", wasm_pack)?;
    Ok(())
}
