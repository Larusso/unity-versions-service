use iron::mime::Mime;
use iron::prelude::*;
use iron::status::Status;
use router::Router;

macro_rules! version_result {
    ($hash:expr) => {{
        let content_type = "application/json".parse::<Mime>().unwrap();
        Ok(Response::with((
            content_type,
            Status::Ok,
            json!($hash).to_string(),
        )))
    }};
}

macro_rules! versions_routes {
    ($router:ident, $($version:expr => $hash:expr),+ $(,)*) => ({
        $(
            trace!("{} -> {}", $version, $hash);
            $router.get(format!("/versions/{}/hash",$version), |_r: &mut Request| version_result!($hash), $version);
        )*
    });
}

include!(concat!(env!("OUT_DIR"), "/version_routes.rs"));
include!(concat!(env!("OUT_DIR"), "/compatible_version_routes.rs"));

pub fn add_version_routes(router:&mut Router) {
    _add_version_routes(router);
    _add_compatible_version_routes(router);
}

include!(concat!(env!("OUT_DIR"), "/versions.rs"));

fn versions_handler(_r: &mut Request) -> IronResult<Response> {
    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((
        content_type,
        Status::Ok,
        VERSIONS,
    )))
}

pub fn add_versions_route(router:&mut Router) {
    router.get("versions/", versions_handler, "versions");
}


// pub fn add_compatible_version_route(router:&mut Router) {
//     router.get("version/")
// }
