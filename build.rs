use std::env;
use std::process::Command;

use ignore::Walk;

fn main() {
    build_wasm_frontend();
}

/// Build the wasm component used for the front end of the website.
/// Requires `wasm-pack` CLI to be present and installed on the
/// your system.
///
/// Runs the command `wasm-pack build --target web --out-dir ../public/js/gui`
fn build_wasm_frontend() {
    // ensure that this script runs every time something within the gui crate changes
    println!("cargo:rerun-if-changed=gui/Cargo.lock");
    for result in Walk::new("gui/") {
        match result {
            Ok(entry) => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }

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

    let child = command.spawn().expect("wasm-pack build command failed");
    let output = child
        .wait_with_output()
        .expect("failed to wait for child process");
    assert!(output.status.success());

    // enable panic here for debugging due to a stupid feature where
    // stdout from this module isn't even included in cargo build -vv.
    // panic!("debugging");
}
