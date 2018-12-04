extern crate base64;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate cli_core;
extern crate log;
extern crate regex;
extern crate serde_json;
extern crate serde_yaml;
extern crate tempfile;

use serde::Serialize;
use std::collections::BTreeMap;
use serde::Serializer;
use cli_core::style;
use cli_core::ColorOption;
use log::{debug, info, warn};
use regex::Regex;
use std::collections::HashMap;
use std::env::var;
use std::io;
use std::path::Path;
use std::ops::Deref;
use std::ops::DerefMut;

const USAGE: &str = "
update-versions - Fetch latest versions and update versions.yml on repo.

Usage:
  update-versions [options]
  update-versions (-h | --help)

Options:
  --token=TOKEN         a github auth token
  --message=MESSAGE     the commit message to use
  --repo-name=REPO      name of the repo
  --repo-owner=OWNER    owner of the github repo
  -f, --force           force refresh of the list
  -v, --verbose         print more output
  -d, --debug           print debug output
  --color WHEN          Coloring: auto, always, never [default: auto]
  -h, --help            show this help message and exit
";

#[derive(Debug, Deserialize)]
pub struct Settings {
    flag_token: Option<String>,
    flag_message: Option<String>,
    flag_repo_name: Option<String>,
    flag_repo_owner: Option<String>,
    flag_force: bool,
    flag_verbose: bool,
    flag_debug: bool,
    flag_color: ColorOption,
}

impl Settings {
    pub fn token(&self) -> io::Result<String> {
        self.flag_token
            .clone()
            .or_else(|| var("UVM_VERSION_UPDATE_TOKEN").ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No token provided"))
    }

    pub fn message(&self) -> String {
        self.flag_message
            .clone()
            .or_else(|| var("UVM_VERSION_UPDATE_MESSAGE").ok())
            .unwrap_or_else(|| "Update Unity versions".to_string())
    }

    pub fn repo_name(&self) -> io::Result<String> {
        self.flag_repo_name
            .clone()
            .or_else(|| var("UVM_VERSION_UPDATE_REPO_NAME").ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No repo name provided"))
    }

    pub fn repo_owner(&self) -> io::Result<String> {
        self.flag_repo_name
            .clone()
            .or_else(|| var("UVM_VERSION_UPDATE_REPO_OWNER").ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No repo owner provided"))
    }

    pub fn force_update(&self) -> bool {
        self.flag_force
    }
}

impl cli_core::Options for Settings {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn debug(&self) -> bool {
        self.flag_debug
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}

mod github {
    use log::debug;
    use reqwest::header::{ACCEPT, USER_AGENT};
    use std::io;
    use std::io::Read;
    use std::path::Path;

    const API_URL: &str = "https://api.github.com";

    pub struct Github {
        client: reqwest::Client,
        token: String,
        api_url: reqwest::Url,
    }

    #[derive(Debug, Serialize)]
    struct ContentPutInput {
        message: String,
        content: String,
        sha: String,
    }

    #[derive(Debug, Deserialize)]
    struct ContenResult {
        sha: String,
    }

    impl Github {
        pub fn client(token: String, api_url: Option<reqwest::Url>) -> Github {
            let api_url = api_url.unwrap_or_else(|| reqwest::Url::parse(API_URL).unwrap());
            let client = reqwest::Client::new();

            Github {
                client,
                token,
                api_url,
            }
        }

        fn content_url<P: AsRef<Path>>(
            &self,
            repo: &str,
            owner: &str,
            path: P,
        ) -> io::Result<reqwest::Url> {
            let path = path.as_ref().display().to_string();
            let url = self
                .api_url
                .join("repos/")
                .and_then(|url| url.join(&format!("{}/", owner)))
                .and_then(|url| url.join(&format!("{}/", repo)))
                .and_then(|url| url.join("contents/"))
                .and_then(|url| url.join(&path))
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to create URL"))?;

            Ok(url)
        }

        // GET /repos/:owner/:repo/contents/:path
        pub fn get_content_raw<P: AsRef<Path>>(
            &self,
            repo: &str,
            owner: &str,
            path: P,
        ) -> io::Result<reqwest::Response> {
            let path = path.as_ref().display().to_string();
            let url = self.content_url(repo, owner, &path)?;

            let response = self
                .client
                .get(url)
                .header(ACCEPT, "application/vnd.github.VERSION.raw")
                .header(USER_AGENT, "unity-versions-service")
                .bearer_auth(self.token.clone())
                .send()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to load content"))?;
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Download failed for {} with status {}", path, status),
                ));
            }

            debug!("server responds with code {}", status);

            Ok(response)
        }

        fn get_content_sha<P: AsRef<Path>>(
            &self,
            repo: &str,
            owner: &str,
            path: P,
        ) -> io::Result<String> {
            let path = path.as_ref().display().to_string();
            let url = self.content_url(repo, owner, &path)?;

            let response = self
                .client
                .get(url)
                .header(USER_AGENT, "unity-versions-service")
                .bearer_auth(self.token.clone())
                .send()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to load content"))?;
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Download SHA failed for {} with status {}", path, status),
                ));
            }

            let content: ContenResult = serde_json::from_reader(response)?;

            Ok(content.sha)
        }

        // PUT /repos/:owner/:repo/contents/:path
        pub fn put_content<P, C>(
            &self,
            repo: &str,
            owner: &str,
            path: P,
            mut content: C,
            message: String,
        ) -> io::Result<()>
        where
            P: AsRef<Path>,
            C: Read,
        {
            use base64::encode;
            use std::str::from_utf8;

            let path = path.as_ref().display().to_string();
            let url = self.content_url(repo, owner, &path)?;

            let mut bytes = Vec::new();
            content.read_to_end(&mut bytes)?;
            let versions =
                from_utf8(&bytes).map_err(|_| io::Error::new(io::ErrorKind::Other, "Error 1"))?;
            let content = encode(versions);
            let sha = self.get_content_sha(repo, owner, &path)?;

            let input = ContentPutInput {
                message,
                content,
                sha,
            };

            let response = self
                .client
                .put(url)
                .header(USER_AGENT, "unity-versions-service")
                .bearer_auth(self.token.clone())
                .json(&input)
                .send()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to put content"))?;

            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Put content failed for {} with status {}", path, status),
                ));
            }

            Ok(())
        }
    }
}

const UPDATE_STREAM: &str = "https://public-cdn.cloud.unity3d.com/hub/prod/releases-darwin.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamVersion {
    version: String,
    lts: bool,
    download_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stream {
    official: Vec<StreamVersion>,
    beta: Vec<StreamVersion>,
}

impl Stream {
    pub fn into_versions(self) -> impl Iterator<Item = StreamVersion> {
        self.official.into_iter().chain(self.beta.into_iter())
    }
}

type UnityRelease = (String, String);

#[derive(Serialize, Deserialize, Default)]
#[serde(transparent)]
struct VersionsMap {
    #[serde(serialize_with = "ordered_map")]
    map: HashMap<String, String>,
}

impl Deref for VersionsMap {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for VersionsMap {
    fn deref_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.map
    }
}

fn ordered_map<S>(value: &HashMap<String, String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

fn main() -> io::Result<()> {
    use std::fmt::Write;

    let settings: Settings = cli_core::get_options(USAGE)?;
    let token = settings.token()?;
    let repo = settings.repo_name()?;
    let owner = settings.repo_owner()?;
    let mut message = settings.message();
    writeln!(message, "");

    info!("update unity versions");

    let version_stream = download_stream()?;
    let u: Stream = serde_json::from_reader(version_stream)?;

    debug!("{:?}", u.official);
    debug!("{:?}", u.beta);

    let versions = u
        .into_versions()
        .filter_map(|v: StreamVersion| read_version_hash(&v.download_url).ok());

    let github = github::Github::client(token, None);
    let content = github.get_content_raw(&repo, &owner, Path::new("versions.yml"))?;

    let mut remote_versions: VersionsMap = serde_yaml::from_reader(content)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to parse versions"))?;

    let mut has_changes = settings.force_update();
    for version in versions {
        if remote_versions
            .insert(version.0.clone(), version.1)
            .is_none()
        {
            info!("add new version {}", &version.0);
            writeln!(message, "* [ADD] version {}", &version.0);
            has_changes = true
        }
    }

    if has_changes {
        let content = serde_yaml::to_string(&remote_versions)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to create versions"))?;
        github.put_content(
            &repo,
            &owner,
            Path::new("versions.yml"),
            content.as_bytes(),
            message,
        )?;
    } else {
        warn!("No changes");
    }

    println!("{}", style("Finish").green());
    Ok(())
}

fn download_stream() -> io::Result<reqwest::Response> {
    let response = reqwest::get(UPDATE_STREAM)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to download stream json"))?;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Download failed for {} with status {}",
                UPDATE_STREAM, status
            ),
        ));
    }
    Ok(response)
}

//https://download.unity3d.com/download_unity/9758a36cfaa6/MacEditorInstaller/Unity-2017.1.5f1.pkg
fn read_version_hash(url: &str) -> io::Result<UnityRelease> {
    let pattern = Regex::new(r"download(_unity)?/(?P<hash>.*)/MacEditorInstaller/Unity-(?P<version>.*).pkg").unwrap();
    match pattern.captures(&url) {
        Some(caps) => {
            let hash: String = caps.name("hash").map(|m| m.as_str()).unwrap().to_string();
            let version: String = caps.name("version").map(|m| m.as_str()).unwrap().to_string();
            info!("fetched version: {} with hash: {}", &version, &hash);
            Ok((version, hash))
        }
        None => Err(io::Error::new(io::ErrorKind::Other, "failed to read hash")),
    }
}
