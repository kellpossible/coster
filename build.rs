use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    build_wasm_frontend();
}

/// Build the wasm component used for the front end of the website.
/// Requires `wasm-pack` CLI to be present and installed on the
/// your system.
fn build_wasm_frontend() {
    let profile: String = env::var("PROFILE").unwrap();

    // // TODO: switch this depending on the profile, release needs to be
    // // in a place that rust-embed can find it.
    // let wasm_out_dir = Path::new("../target")
    // .join(&profile)
    // .join("gui/pkg");
    // wasm_out_dir.to_str().expect("a valid UTF8 output directory path string")

    let mut command = Command::new("wasm-pack");
    command.current_dir("./gui");
    command.args(&["build", "--target", "web", "--out-dir", "../public/js/gui"]);

    if profile == "debug" {
        command.arg("--dev");
    }

    command.spawn().expect("wasm-pack build command failed");
}