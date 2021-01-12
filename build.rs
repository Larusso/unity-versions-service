extern crate serde;
extern crate serde_yaml;
extern crate serde_json;

use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::io;

fn main() -> io::Result<()> {
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let versions_path = Path::new(&project_dir).join("versions.yml");
    let versions = File::open(&versions_path).unwrap();
    let version_list: BTreeMap<String, String> = serde_yaml::from_reader(versions).unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let routes_path = Path::new(&out_dir).join("version_routes.rs");
    let versions_path = Path::new(&out_dir).join("versions.rs");

    let mut f = File::create(&routes_path).unwrap();

    writeln!(f, "pub fn _add_version_routes(router:&mut Router) {{")?;
    writeln!(f, r#"    debug!("add unity version routes");"#)?;
    writeln!(f, r#"    versions_routes!(router,"#)?;
    for item in &version_list {
        writeln!(f, r#"        "{}" => "{}","#, item.0, item.1)?;
    }
    writeln!(f, "    );")?;
    writeln!(f, "}}")?;

    let mut f = File::create(&versions_path).unwrap();

    writeln!(f, "const VERSIONS: &str = r#\"")?;
    writeln!(f, "{}", serde_json::to_string_pretty(&version_list).unwrap())?;
    writeln!(f, "\"#;")?;
    Ok(())
}
