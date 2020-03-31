use crate::gettext::GettextConfig;

use std::fs::read_to_string;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_derive::Deserialize;
use toml;
use tr::tr;

pub struct Crate {
    pub name: String,
    pub path: Box<Path>,
}

impl Crate {
    pub fn new<S: Into<String>>(name: S, path: Box<Path>) -> Crate {
        Crate {
            name: name.into(),
            path,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct I18nConfig {
    pub src_locale: String,
    pub locales: Vec<String>,
    pub gettext_config: Option<GettextConfig>,
}

impl I18nConfig {
    pub fn gettext_config(&self) -> Result<&GettextConfig> {
        match &self.gettext_config {
            Some(config) => Ok(config),
            None => Err(anyhow!(tr!(
                "there is no gettext config available in this i18n config"
            ))),
        }
    }
}

pub fn read_config() -> Result<I18nConfig> {
    let toml_path = Path::new("i18n.toml");
    let toml_str = read_to_string(toml_path).context("trouble reading i18n.toml")?;
    let config: I18nConfig =
        toml::from_str(toml_str.as_ref()).context("trouble parsing i18n.toml")?;
    Ok(config)
}
