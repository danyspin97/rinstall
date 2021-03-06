use std::str::FromStr;

use camino::Utf8PathBuf;
use color_eyre::{eyre::ensure, Result};
use serde::Deserialize;
use void::Void;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Icon {
    #[serde(rename(deserialize = "src"))]
    pub source: Utf8PathBuf,
    #[serde(rename(deserialize = "dst"))]
    pub destination: Option<Utf8PathBuf>,
    #[serde(rename(deserialize = "type"))]
    pub icon_type: Option<String>,
    pub theme: Option<String>,
    pub dimensions: Option<String>,
    #[serde(default)]
    pub pixmaps: bool,
}

impl Icon {
    const fn new_with_source(source: Utf8PathBuf) -> Self {
        Self {
            source,
            destination: None,
            icon_type: None,
            theme: None,
            dimensions: None,
            pixmaps: true,
        }
    }

    pub fn get_destination(&self) -> Result<Utf8PathBuf> {
        let dest = if self.pixmaps {
            Utf8PathBuf::from("pixmaps").join("")
        } else {
            ensure!(
                self.dimensions.is_some(),
                "dimensions must be set for all non pixmaps icons"
            );

            let default_theme: &'static str = "hicolor";
            let default_icon_type: &'static str = "apps";
            let theme = self.theme.as_deref().unwrap_or(default_theme);
            let icon_type = self.icon_type.as_deref().unwrap_or(default_icon_type);

            Utf8PathBuf::from("icons")
                .join(theme)
                .join(self.dimensions.as_ref().unwrap())
                .join(icon_type)
                .join("")
        };

        if let Some(destination) = &self.destination {
            Ok(dest.join(destination))
        } else {
            Ok(dest)
        }
    }
}

impl FromStr for Icon {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new_with_source(Utf8PathBuf::from(s)))
    }
}
