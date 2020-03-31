use std::env;
use std::ffi::OsStr;
use std::fs::{create_dir_all, read_to_string, remove_file, File, FileType};
use std::io;
use std::path::Path;
use std::{fmt::Display, process::Command};

use anyhow::{anyhow, ensure, Context, Result};
use ignore::Walk;
use serde_derive::Deserialize;
use subprocess::Exec;
use thiserror::Error;
use toml;
use tr::tr;

fn main() {
    build_wasm_frontend().expect("error while building the wasm front-end");
}

#[derive(Debug)]
enum PathType {
    File,
    Directory,
    Symlink,
}

impl Display for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathType::File => write!(f, "file"),
            PathType::Directory => write!(f, "directory"),
            PathType::Symlink => write!(f, "symbolic link"),
        }
    }
}

#[derive(Debug)]
enum PathErrorKind {
    NotValidUTF8 {
        for_item: String,
        path_type: PathType,
    },
    DoesNotExist,
    CannotCreateDirectory(io::Error),
    NotInsideDirectory(String, Box<Path>),
}

#[derive(Error, Debug)]
struct PathError {
    pub path: Box<Path>,
    pub kind: PathErrorKind,
}

impl PathError {
    fn cannot_create_dir<P: Into<Box<Path>>>(path: P, source: io::Error) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::CannotCreateDirectory(source),
        }
    }

    fn does_not_exist<P: Into<Box<Path>>>(path: P) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::DoesNotExist,
        }
    }

    fn not_valid_utf8<F: Into<String>, P: Into<Box<Path>>>(
        path: P,
        for_item: F,
        path_type: PathType,
    ) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::NotValidUTF8 {
                for_item: for_item.into(),
                path_type,
            },
        }
    }

    fn not_inside_dir<S: Into<String>, P: Into<Box<Path>>>(
        path: P,
        parent_name: S,
        parent_path: P,
    ) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::NotInsideDirectory(parent_name.into(), parent_path.into()),
        }
    }
}

impl Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.kind {
            PathErrorKind::NotValidUTF8 {
                for_item,
                path_type,
            } => {
                // {0} is the file path, {1} is the item which it is for, {2} is the type of item (file, directory, etc)
                tr!(
                    "path ({0}) for {1} {2} does not have valid a utf-8 encoding",
                    self.path.to_string_lossy(),
                    for_item,
                    path_type
                )
            }
            PathErrorKind::DoesNotExist => tr!(
                "the path {0} does not exist on the filesystem",
                self.path.to_string_lossy()
            ),
            PathErrorKind::CannotCreateDirectory(source) => tr!(
                "cannot create the directory {0} because: {1}",
                self.path.to_string_lossy(),
                source
            ),
            PathErrorKind::NotInsideDirectory(parent_name, parent_dir) => tr!(
                "the path {0} is not inside the {1} directory {2}",
                self.path.to_string_lossy(),
                parent_name,
                parent_dir.to_string_lossy(),
            ),
        };

        write!(f, "{}", message)
    }
}

#[derive(Deserialize)]
struct BuildConfig {
    i18n: Option<I18nConfig>,
}

#[derive(Deserialize)]
struct I18nConfig {
    src_locale: String,
    locales: Vec<String>,
    crates: Vec<String>,
    xtr: Option<bool>,
}

fn read_toml_config() -> Result<BuildConfig> {
    let toml_path = Path::new("Build.toml");
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
    ensure_gui_watch_rerun()?;

    match config.i18n {
        Some(i18n_config) => {
            build_i18n(&i18n_config)?;
        }
        None => {}
    }
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

fn check_path_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        Err(anyhow!(PathError::does_not_exist(path)))
    } else {
        Ok(())
    }
}

fn create_dir_all_if_not_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_dir_all(path.clone()).map_err(|e| PathError::cannot_create_dir(path.clone(), e))?;
    }
    Ok(())
}

fn i18n_xtr(crate_name: &str, src_dir: &Path, pot_dir: &Path) -> Result<()> {
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
            Err(err) => {
                return Err(anyhow!(
                    "error walking directory {}/src: {}",
                    crate_name,
                    err
                ))
            }
        }
    }

    let mut pot_paths = Vec::new();
    let pot_src_dir = pot_dir.join("src");

    // create pot and pot/tmp if they don't exist
    create_dir_all_if_not_exists(&pot_src_dir)?;

    for rs_file_path in rs_files {
        let parent_dir = rs_file_path.parent().context(format!(
            "the rs file {0} is not inside a directory",
            rs_file_path.to_string_lossy()
        ))?;
        let src_dir_relative = parent_dir.strip_prefix(src_dir).map_err(|_| {
            PathError::not_inside_dir(parent_dir, format!("crate {0}/src", crate_name), src_dir)
        })?;
        let file_stem = rs_file_path.file_stem().context(format!(
            "expected rs file path {0} would have a filename",
            rs_file_path.to_string_lossy()
        ))?;

        let pot_file_path = pot_src_dir
            .join(src_dir_relative)
            .join(file_stem)
            .with_extension("pot");

        let pot_dir = pot_file_path.parent().with_context(|| {
            format!(
                "the pot file {0} is not inside a directory",
                pot_file_path.to_string_lossy()
            )
        })?;
        create_dir_all(pot_dir)?;

        // ======= Run the `xtr` command to extract translatable strings =======
        let xtr_command_name = "xtr";
        let mut xtr = Command::new(xtr_command_name);
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
            pot_file_path.to_str().ok_or(PathError::not_valid_utf8(
                pot_file_path.clone(),
                "pot",
                PathType::File,
            ))?,
            rs_file_path.to_str().ok_or(PathError::not_valid_utf8(
                rs_file_path.clone(),
                "rs",
                PathType::File,
            ))?,
        ]);

        run_command_and_check_success(xtr_command_name, xtr)?;

        pot_paths.push(pot_file_path.to_owned());
    }

    let mut msgcat_args: Vec<Box<OsStr>> = Vec::new();

    for path in pot_paths {
        msgcat_args.push(Box::from(path.as_os_str()));
    }

    let combined_pot_file_path = pot_dir.join("gui.pot");

    if combined_pot_file_path.exists() {
        remove_file(combined_pot_file_path.clone()).context("unable to delete .pot")?;
    }

    let combined_pot_file =
        File::create(combined_pot_file_path).expect("unable to create .pot file");

    // ====== run the `msgcat` command to combine pot files into gui.pot =======
    let msgcat_command_name = "msgcat";
    let msgcat = Exec::cmd(msgcat_command_name)
        .args(msgcat_args.as_slice())
        .stdout(combined_pot_file);

    msgcat.join().context(format!(
        "there was a problem executing the {0} command",
        msgcat_command_name
    ))?;

    Ok(())
}

fn i18n_msginit(
    crate_name: &str,
    i18n_config: &I18nConfig,
    pot_dir: &Path,
    po_dir: &Path,
) -> Result<()> {
    let pot_file_path = pot_dir.join(crate_name).with_extension("pot");

    check_path_exists(&pot_file_path)?;

    create_dir_all_if_not_exists(po_dir)?;

    let msginit_command_name = "msginit";

    for locale in &i18n_config.locales {
        let po_locale_dir = po_dir.join(locale.clone());
        let po_path = po_locale_dir.join(crate_name).with_extension("po");

        if !po_path.exists() {
            create_dir_all(po_locale_dir.clone())
                .map_err(|e| PathError::cannot_create_dir(po_locale_dir, e))?;
            
            let mut msginit = Command::new(msginit_command_name);
            msginit.args(&[
                format!(
                    "--input={}",
                    pot_file_path.to_str().ok_or(PathError::not_valid_utf8(
                        pot_file_path.clone(),
                        "pot",
                        PathType::File,
                    ))?
                ),
                format!("--locale={}.UTF-8", locale),
                format!(
                    "--output={}",
                    po_path.to_str().ok_or(PathError::not_valid_utf8(
                        po_path.clone(),
                        "po",
                        PathType::File,
                    ))?
                ),
            ]);

            run_command_and_check_success(msginit_command_name, msginit)?;
        }
    }

    Ok(())
}

fn i18n_msgmerge(
    crate_name: &str,
    i18n_config: &I18nConfig,
    pot_dir: &Path,
    po_dir: &Path,
) -> Result<()> {
    let pot_file_path = pot_dir.join(crate_name).with_extension("pot");

    check_path_exists(&pot_file_path)?;

    let msgmerge_command_name = "msgmerge";

    for locale in &i18n_config.locales {
        let po_file_path = po_dir.join(locale).join(crate_name).with_extension("po");

        check_path_exists(&po_file_path)?;

        let mut msgmerge = Command::new(msgmerge_command_name);
        msgmerge.args(&[
            "--backup=none",
            "--update",
            po_file_path.to_str().ok_or(PathError::not_valid_utf8(
                po_file_path.clone(),
                "pot",
                PathType::File,
            ))?,
            pot_file_path.to_str().ok_or(PathError::not_valid_utf8(
                pot_file_path.clone(),
                "pot",
                PathType::File,
            ))?,
        ]);

        run_command_and_check_success(msgmerge_command_name, msgmerge)?;
    }

    Ok(())
}

fn i18n_msgfmt(
    crate_name: &str,
    i18n_config: &I18nConfig,
    po_dir: &Path,
    mo_dir: &Path,
) -> Result<()> {
    let msgfmt_command_name = "msgfmt";

    for locale in &i18n_config.locales {
        let po_file_path = po_dir
            .join(locale.clone())
            .join(crate_name)
            .with_extension("po");

            check_path_exists(&po_file_path)?;

        let mo_locale_dir = mo_dir.join(locale);

        if !mo_locale_dir.exists() {
            create_dir_all(mo_locale_dir.clone()).context("trouble creating mo directory")?;
        }

        let mo_file_path = mo_locale_dir.join(crate_name).with_extension("mo");

        let mut msgfmt = Command::new(msgfmt_command_name);
        msgfmt.args(&[
            format!(
                "--output-file={}",
                mo_file_path
                    .to_str()
                    .expect("mo file path is not valid utf-8")
            )
            .as_str(),
            po_file_path
                .to_str()
                .expect("po file path is not valid utf-8"),
        ]);

        run_command_and_check_success(msgfmt_command_name, msgfmt)?;
    }

    Ok(())
}

fn build_i18n(i18n_config: &I18nConfig) -> Result<()> {
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
            i18n_xtr(subcrate.as_str(), src_dir.as_path(), pot_dir.as_path())?;
            i18n_msginit(
                subcrate.as_str(),
                i18n_config,
                pot_dir.as_path(),
                po_dir.as_path(),
            )?;
            i18n_msgmerge(
                subcrate.as_str(),
                i18n_config,
                pot_dir.as_path(),
                po_dir.as_path(),
            )?;
        }

        i18n_msgfmt(
            subcrate.as_str(),
            i18n_config,
            po_dir.as_path(),
            mo_dir.as_path(),
        )?;
    }

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
