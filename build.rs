use std::env;
use std::process::Command;
use std::path::Path;

use ignore::Walk;

fn main() {
    build_wasm_frontend();
}

/// Build the wasm component used for the front end of the website.
/// Requires `wasm-pack` CLI, `xtr`, and GNU Gettext CLI tools
/// `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
/// system path.
///
/// Runs the command `wasm-pack build --target web --out-dir
/// ../public/js/gui`
fn build_wasm_frontend() {
    ensure_gui_watch_rerun();
    build_wasm_i18n();
    build_wasm();

    // enable panic here for debugging due to a stupid feature where
    // stdout from this module isn't even included in cargo build -vv.
    panic!("debugging");
}

/// Ensure that this script runs every time something within the gui
/// crate changes.
fn ensure_gui_watch_rerun() {
    println!("cargo:rerun-if-changed=gui/Cargo.lock");
    for result in Walk::new("gui/") {
        match result {
            Ok(entry) => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
            Err(err) => eprintln!("error walking directory gui: {}", err),
        }
    }
}

fn build_wasm_i18n() {
    // TODO: oops we don't want to be walking over pot files, instead I want to walk over the rs files
    let mut pot_files: Vec<Box<Path>> = Vec::new();

    for result in Walk::new("gui/i18n/pot") {

        match result {
            Ok(entry) => {
                let path = entry.path().clone();
                
                match path.extension() {
                    Some(extension) => {
                        if extension.to_str() == Some("pot") {
                            pot_files.push(Box::from(path))
                        }
                    },
                    None => {}
                }
            },
            Err(err) => eprintln!("error walking directory gui/pot: {}", err)
        }
    }

    println!("pot files {:?}", pot_files);
}

fn build_wasm() {
    let profile: String = env::var("PROFILE").unwrap();

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
}
