use std::env;
use std::ffi::OsStr;
use std::fs::{create_dir_all, read_to_string, remove_file, File};
use std::path::Path;
use std::process::Command;

use ignore::Walk;
use subprocess::Exec;
use toml;
use toml::map::Map;
use serde_derive::Deserialize;

fn main() {
    build_wasm_frontend();
}

#[derive(Deserialize)]
struct BuildConfig {
    i18n: Option<I18nConfig>
}

#[derive(Deserialize)]
struct I18nConfig {
    src_locale: String,
    locales: Vec<String>,
    crates: Vec<String>,
    xtr: Option<bool>,
}

fn read_toml_config() -> BuildConfig{
    let toml_path = Path::new("Build.toml");
    let toml_str = read_to_string(toml_path).expect("trouble reading Cargo.toml");
    let config: BuildConfig = toml::from_str(toml_str.as_ref()).expect("trouble parsing Cargo.toml");
    println!("cargo:rerun-if-changed=Build.toml");
    config
}

/// Build the wasm component used for the front end of the website.
/// Requires `wasm-pack` CLI, `xtr`, and GNU Gettext CLI tools
/// `msginit`, `msgfmt`, `msgmerge` and `msgcat` to be present in your
/// system path.
///
/// Runs the command `wasm-pack build --target web --out-dir
/// ../public/js/gui`
fn build_wasm_frontend() {
    let config = read_toml_config();
    ensure_gui_watch_rerun();

    match config.i18n {
        Some(i18n_config) => {
            build_i18n(&i18n_config);
        },
        None => {},
    }
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

fn i18n_xtr(crate_name: &str, src_dir: &Path, pot_dir: &Path) {
    let mut rs_files: Vec<Box<Path>> = Vec::new();

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
            Err(err) => eprintln!("error walking directory {}/src: {}", crate_name, err),
        }
    }

    let mut pot_paths = Vec::new();
    let pot_src_dir = pot_dir.join("src");

    // create pot and pot/tmp if they don't exist
    if !pot_src_dir.exists() {
        create_dir_all(pot_src_dir.clone()).expect("trouble creating pot/src directory");
    }

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

        let pot_path = pot_src_dir
            .join(src_dir_relative)
            .join(file_stem)
            .with_extension("pot");

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
            "--default-domain",
            crate_name,
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
        remove_file(combined_pot_file_path.clone()).expect("unable to delete .pot");
    }

    let combined_pot_file =
        File::create(combined_pot_file_path).expect("unable to create .pot file");

    // ====== run the `msgcat` command to combine pot files into gui.pot =======
    let msgcat = Exec::cmd("msgcat")
        .args(msgcat_args.as_slice())
        .stdout(combined_pot_file);

    msgcat.join().expect("problem executing msgcat");
}

fn i18n_msginit(crate_name: &str, i18n_config: &I18nConfig, pot_dir: &Path, po_dir: &Path) {
    let pot_file_path = pot_dir.join(crate_name).with_extension("pot");

    if !pot_file_path.exists() {
        panic!(format!("pot file {:?} does not exist", pot_file_path));
    }
    
    if !po_dir.exists() {
        create_dir_all(po_dir.clone()).expect("trouble creating pot directory");
    }
    
    for locale in &i18n_config.locales {
        let po_locale_dir = po_dir.join(locale.clone());
        let po_path = po_locale_dir.join(crate_name).with_extension("po");

        if !po_path.exists() {
            create_dir_all(po_locale_dir).expect("problem creating po locale directory");
            let mut msginit = Command::new("msginit");
            msginit.args(&[
                format!("--input={}", pot_file_path.to_str().expect("pot file path is not valid utf-8")),
                format!("--locale={}.UTF-8", locale),
                format!("--output={}", po_path.to_str().expect("po file path is not valid utf-8")),
            ]);

            let output = msginit
                .spawn()
                .expect("msginit command failed")
                .wait_with_output()
                .expect("failed to wait for msginit command completion");

            assert!(output.status.success());
        }
    }
}

fn i18n_msgmerge(crate_name: &str, i18n_config: &I18nConfig, pot_dir: &Path, po_dir: &Path) {
    let pot_file_path = pot_dir.join(crate_name).with_extension("pot");

    if !pot_file_path.exists() {
        panic!(format!("pot file {:?} does not exist", pot_file_path));
    }

    for locale in &i18n_config.locales {
        let po_file_path = po_dir.join(locale).join(crate_name).with_extension("po");

        if !po_file_path.exists() {
            panic!(format!("po file {:?} does not exist", po_file_path));
        }

        println!("updating: {:?}", po_file_path);

        let mut msgmerge = Command::new("msgmerge");
        msgmerge.args(&[
            "--backup=none",
            "--update",
            po_file_path.to_str().expect("po file path is not valid utf-8"),
            pot_file_path.to_str().expect("pot; file path is not valid utf-8"),
        ]);

        let output = msgmerge
            .spawn()
            .expect("msgmerge command failed")
            .wait_with_output()
            .expect("failed to wait for msgmerge command completion");

        assert!(output.status.success());
    }
}

fn i18n_msgfmt(crate_name: &str, i18n_config: &I18nConfig, po_dir: &Path, mo_dir: &Path) {
    for locale in &i18n_config.locales {
        let po_file_path = po_dir.join(locale.clone()).join(crate_name).with_extension("po");

        if !po_file_path.exists() {
            panic!(format!("po file {:?} does not exist", po_file_path));
        }

        let mo_locale_dir = mo_dir.join(locale);

        if !mo_locale_dir.exists() {
            create_dir_all(mo_locale_dir.clone()).expect("trouble creating mo directory");
        }

        let mo_file_path = mo_locale_dir.join(crate_name).with_extension("mo");

        let mut msgfmt = Command::new("msgfmt");
        msgfmt.args(&[
            format!("--output-file={}", mo_file_path.to_str().expect("mo file path is not valid utf-8")).as_str(),
            po_file_path.to_str().expect("po file path is not valid utf-8"),
        ]);

        let output = msgfmt
            .spawn()
            .expect("msgfmt command failed")
            .wait_with_output()
            .expect("failed to wait for msgfmt command completion");

        assert!(output.status.success());
    }
}

fn build_i18n(i18n_config: &I18nConfig) {
    let do_xtr = match i18n_config.xtr {
        Some(xtr_value) => xtr_value,
        None => true,
    };

    for subcrate in &i18n_config.crates {
        let crate_dir = Path::new(subcrate.as_str());
        let i18n_dir = crate_dir.join("i18n");
        let src_dir = crate_dir.join("src");
        let pot_dir = i18n_dir.join("pot");
        let po_dir = i18n_dir.join("po");
        let mo_dir = i18n_dir.join("mo");

        if do_xtr {
            i18n_xtr(subcrate.as_str(), src_dir.as_path(), pot_dir.as_path());
            i18n_msginit(subcrate.as_str(), i18n_config, pot_dir.as_path(), po_dir.as_path());
            i18n_msgmerge(subcrate.as_str(), i18n_config, pot_dir.as_path(), po_dir.as_path());
        }

        i18n_msgfmt(subcrate.as_str(), i18n_config, po_dir.as_path(), mo_dir.as_path())
    }
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
