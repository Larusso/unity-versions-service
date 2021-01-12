use cli_core::{ColorOption, Options};
use serde::Deserialize;
use std::env;

const USAGE: &str = "
unity-versions-service - A simple webserver to deliver unity version information

Usage:
  unity-versions-service [options]
  unity-versions-service (-h | --help)

Options:
  --port=PORT       the server port number
  -v, --verbose     print more output
  -d, --debug       print debug output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

#[derive(Debug, Deserialize)]
pub struct Settings {
    flag_port: Option<u16>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_color: ColorOption,
}

impl Settings {
    pub fn port(&self) -> u16 {
        self.flag_port.or_else(|| self.env_port()).unwrap_or(8080)
    }

    fn env_port(&self) -> Option<u16> {
        env::var("PORT").ok().and_then(|p| p.parse().ok())
    }
}

impl Options for Settings {
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

fn main() -> std::io::Result<()> {
    let options: Settings = cli_core::get_options(USAGE)?;
    unity_versions_service::start_server(("0.0.0.0", options.port())).unwrap();
    Ok(())
}
