use std::env;
use std::ffi::OsStr;
use std::fs::{create_dir_all, remove_file, read_to_string};
use std::path::Path;
use std::process::Command;
use toml;
use ignore::Walk;
use subprocess::Exec;

fn main() {
    build_wasm_frontend();
}

fn read_toml_config() -> toml::Value {
    let toml_str = read_to_string("Cargo.coml").expect("trouble reading Cargo.toml");
    toml::from_str(toml_str.as_ref()).expect("trouble parsing Cargo.toml")
}

/// Build the wasm component used for the front end of the website.
/// Requires `wasm-pack` CLI, `xtr`, and GNU Gettext CLI tools
/// `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
/// system path.
///
/// Runs the command `wasm-pack build --target web --out-dir
/// ../public/js/gui`
fn build_wasm_frontend() {
    let toml_config = read_toml_config();
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
    let mut rs_files: Vec<Box<Path>> = Vec::new();

    let src_dir = Path::new("gui/src");
    for result in Walk::new(src_dir) {
        match result {
            Ok(entry) => {
                let path = entry.path().clone();

                match path.extension() {
                    Some(extension) => {
                        if extension.to_str() == Some("rs") {
                            rs_files.push(Box::from(path))
                        }
                    }
                    None => {}
                }
            }
            Err(err) => eprintln!("error walking directory gui/src: {}", err),
        }
    }

    let i18n_dir = Path::new("gui/i18n");
    let pot_dir = i18n_dir.join("pot");
    let pot_tmp_dir = pot_dir.join("tmp");

    if !pot_tmp_dir.exists() {
        create_dir_all(pot_tmp_dir.clone()).expect("trouble creating gui/pot/tmp directory");
    }

    let mut pot_paths = Vec::new();

    for path in rs_files {
        let parent_dir = path
            .parent()
            .expect("expected there to be a parent directory for the rs file");
        let src_dir_relative = parent_dir
            .strip_prefix(src_dir)
            .expect("expected parent_dir to be a superset of src_dir");
        let file_stem = path
            .file_stem()
            .expect("expected rs file path would have a filename");

        let pot_path = pot_tmp_dir
            .join(src_dir_relative)
            .join(file_stem)
            .with_extension("pot");
        println!("pot_path: {:?}", pot_path);
        println!("path: {:?}", path);

        let pot_dir = pot_path
            .parent()
            .expect("pot file will have a parent directory");
        create_dir_all(pot_dir).expect("unable to create directory");

        // ======= Run the `xtr` command to extract translatable strings =======
        let mut xtr = Command::new("xtr");
        xtr.args(&[
            "--package-name",
            "Coster",
            "--package-version",
            "0.1", //TODO: replace this with version from TOML
            "--copyright-holder",
            "Luke Frisken",
            "--msgid-bugs-address",
            "l.frisken@gmail.com",
            "-o",
            pot_path.to_str().expect("path isn't valid unicode"),
            path.to_str().expect("path isn't valid unicode"),
        ]);
        let output = xtr
            .spawn()
            .expect("xtr command failed")
            .wait_with_output()
            .expect("failed to wait for xtr command completion");

        assert!(output.status.success());

        pot_paths.push(pot_path.to_owned());
    }

    let mut msgcat_args: Vec<Box<OsStr>> = Vec::new();

    for path in pot_paths {
        msgcat_args.push(Box::from(path.as_os_str()));
    }

    let combined_pot_file_path = pot_dir.join("gui.pot");

    if combined_pot_file_path.exists() {
        remove_file(combined_pot_file_path.clone()).expect("unable to delete gui.pot");
    }

    let combined_pot_file =
        File::create(combined_pot_file_path).expect("unable to create gui.pot file");

    // ====== run the `msgcat` command to combine pot files into gui.pot =======
    let msgcat = Exec::cmd("msgcat")
        .args(msgcat_args.as_slice())
        .stdout(combined_pot_file);

    let msgcat_out = msgcat
        .capture()
        .expect("problem executing msgcat")
        .stdout_str();


    
}

fn build_wasm() {
    let profile: String = env::var("PROFILE").unwrap();

    let mut wasm_pack = Command::new("wasm-pack");
    wasm_pack.current_dir("./gui");
    wasm_pack.args(&["build", "--target", "web", "--out-dir", "../public/js/gui"]);

    if profile == "debug" {
        wasm_pack.arg("--dev");
    }

    let child = wasm_pack.spawn().expect("wasm-pack build command failed");
    let output = child
        .wait_with_output()
        .expect("failed to wait for child process");
    assert!(output.status.success());
}
