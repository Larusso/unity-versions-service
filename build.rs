extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate uvm_core;

use semver::VersionReq;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use uvm_core::unity::fetch_matching_version;
use uvm_core::unity::Version;
use uvm_core::unity::VersionType;

fn main() {
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let versions_path = Path::new(&project_dir).join("versions.yml");
    let versions = File::open(&versions_path).unwrap();
    let version_list: BTreeMap<String, String> = serde_yaml::from_reader(versions).unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let routes_path = Path::new(&out_dir).join("version_routes.rs");
    let versions_path = Path::new(&out_dir).join("versions.rs");
    let compatible_routes_path = Path::new(&out_dir).join("compatible_version_routes.rs");

    let mut f = File::create(&routes_path).unwrap();
    write_versions_route(&mut f, &version_list);

    let mut f = File::create(&versions_path).unwrap();
    write_version_content(&mut f, &version_list);

    let mut f = File::create(&compatible_routes_path).unwrap();
    write_compatible_routes(&mut f, &version_list);
}

fn write_versions_route(buffer: &mut File, version_list: &BTreeMap<String, String>) {
    writeln!(buffer, "pub fn _add_version_routes(router:&mut Router) {{");
    writeln!(buffer, r#"    debug!("add unity version routes");"#);
    writeln!(buffer, r#"    versions_routes!(router,"#);
    for item in version_list {
        writeln!(buffer, r#"        "{}" => "{}","#, item.0, item.1);
    }
    writeln!(buffer, "    );");
    writeln!(buffer, "}}");
}

fn write_version_content(buffer: &mut File, version_list: &BTreeMap<String, String>) {
    writeln!(buffer, "const VERSIONS: &str = r#\"");
    writeln!(
        buffer,
        "{}",
        serde_json::to_string_pretty(version_list).unwrap()
    );
    writeln!(buffer, "\"#;");
}

fn write_compatible_routes<W: Write>(buffer: &mut W, version_list: &BTreeMap<String, String>) {
    writeln!(
        buffer,
        "pub fn _add_compatible_version_routes(router:&mut Router) {{"
    );

    // convert versions to unity versions
    let versions: Vec<Version> = version_list
        .keys()
        .map(|raw_version| Version::from_str(&raw_version).unwrap())
        .collect();

    // filter only major versions: 2018, 2019, etc
    let major_versions: HashSet<u64> = versions.clone().into_iter().map(|v| v.major()).collect();
    for major in major_versions {
        for t in vec![
            VersionType::Final,
            VersionType::Patch,
            VersionType::Beta,
            VersionType::Alpha,
        ] {
            let req = VersionReq::from_str(&format!("~{}", major)).unwrap();
            if let Ok(compatible_version) =
                fetch_matching_version(versions.clone().into_iter(), req, t)
            {
                if t == VersionType::Final {
                    writeln!(buffer, r#"        router.get("/version/{version}", |_r: &mut Request| version_result!("{compatible_version}"), "comp_version_{version}");"#, version = major,compatible_version = compatible_version);
                }
                writeln!(buffer, r#"        router.get("/version/{version}/{type:#}", |_r: &mut Request| version_result!("{compatible_version}"), "comp_version_{version}_{type}");"#, version = major, type = t,compatible_version = compatible_version);
            }
        }
    }

    // filter all major.minor combinations: 2018.1, 2018.2, etc
    let major_minor_versions: HashSet<(u64, u64)> = versions
        .clone()
        .into_iter()
        .map(|v| (v.major(), v.minor()))
        .collect();
    for major_minor in major_minor_versions {
        for t in vec![
            VersionType::Final,
            VersionType::Patch,
            VersionType::Beta,
            VersionType::Alpha,
        ] {
            let req =
                VersionReq::from_str(&format!("~{}.{}", major_minor.0, major_minor.1)).unwrap();
            if let Ok(compatible_version) =
                fetch_matching_version(versions.clone().into_iter(), req, t)
            {
                if t == VersionType::Final {
                    writeln!(buffer, r#"        router.get("/version/{version}", |_r: &mut Request| version_result!("{compatible_version}"), "comp_version_{version}");"#, version = format!("{}.{}", major_minor.0, major_minor.1),compatible_version = compatible_version);
                }
                writeln!(buffer, r#"        router.get("/version/{version}/{type:#}", |_r: &mut Request| version_result!("{compatible_version}"), "comp_version_{version}_{type}");"#, version = format!("{}.{}", major_minor.0, major_minor.1), type = t,compatible_version = compatible_version);
            }
        }
    }
    // generate for all major.minor.patch combinations
    for version in versions.clone() {
        for t in vec![
            VersionType::Final,
            VersionType::Patch,
            VersionType::Beta,
            VersionType::Alpha,
        ] {
            let req = VersionReq::from_str(&format!("~{}", version.base())).unwrap();
            if let Ok(compatible_version) =
                fetch_matching_version(versions.clone().into_iter(), req, t)
            {
                if t == VersionType::Final {
                    writeln!(buffer, r#"        router.get("/version/{version}", |_r: &mut Request| version_result!("{compatible_version}"), "comp_version_{version}");"#, version = version.base(),compatible_version = compatible_version);
                }
                writeln!(buffer, r#"        router.get("/version/{version}/{type:#}", |_r: &mut Request| version_result!("{compatible_version}"), "comp_version_{version}_{type}");"#, version = version.base(), type = t,compatible_version = compatible_version);
            }
        }
    }

    writeln!(buffer, "}}");
}
