extern crate serde;
extern crate serde_yaml;

use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let versions_path = Path::new(&project_dir).join("versions.yml");
    let versions = File::open(&versions_path).unwrap();
    let version_list: BTreeMap<String, String> = serde_yaml::from_reader(versions).unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version_routes.rs");
    let mut f = File::create(&dest_path).unwrap();

    writeln!(f, "pub fn add_version_routes(router:&mut Router) {{");
    writeln!(f, r#"    debug!("add unity version routes");"#);
    for item in version_list {
        writeln!(f, r#"    trace!("{} -> {}");"#, item.0, item.1);
        writeln!(f, r#"    router.get("/versions/{}/hash", |_r: &mut Request| version_result!("{}"), "{}");"#, item.0, item.1, item.0);
    }
    writeln!(f, "}}");
}
