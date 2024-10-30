use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::toml::{read_toml_file, write_toml_file, TomlFileError};

static RELEASE_VERSION: Option<&'static str> = option_env!("JOLTEON_RELEASE_VERSION");
static RELEASES_URL: &str = "https://api.github.com/repos/lautarodragan/jolteon/releases?per_page=3";

#[derive(Serialize, Deserialize, Debug)]
pub struct Release {
    url: String,
    assets_url: String,
    tag_name: String,
    target_commitish: String,
    name: String,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: String,
    body: String,
    tarball_url: String,
    zipball_url: String,
    // assets: Vec<...>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Releases {
    pub releases: Vec<Release>,
}

#[derive(Debug)]
pub enum ReleasesError {
    Reqwest(reqwest::Error),
    TomlFileError(TomlFileError),
}

impl From<TomlFileError> for ReleasesError {
    fn from(value: TomlFileError) -> Self {
        ReleasesError::TomlFileError(value)
    }
}

impl From<reqwest::Error> for ReleasesError {
    fn from(value: reqwest::Error) -> Self {
        ReleasesError::Reqwest(value)
    }
}

pub fn get_releases() -> Result<(), ReleasesError> {
    let client = Client::new();

    let body = client
        .get(RELEASES_URL)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "jolteon")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .timeout(Duration::from_secs(5));

    let body = body.send()?;
    let releases: Vec<Release> = body.json()?;

    log::debug!(target: "::get_releases", "Got releases = {releases:#?}");

    let releases = Releases {
       releases,
    };

    write_toml_file("releases", &releases)?;

    Ok(())
}

pub fn can_i_has_rls() -> Result<(), ReleasesError> {
    let target = "::can_i_has_rls";

    log::trace!(target: target, "RELEASE_VERSION: {RELEASE_VERSION:?}");

    let releases: Releases = read_toml_file("releases")?;

    log::trace!(target: target, "we has rls file = {releases:#?}");

    log::trace!(target: target, "we has rls published_at = {:?}", releases.releases.get(0).map(|r| r.published_at.as_str()));


    Ok(())
}
