#[macro_use]
extern crate log;
extern crate cli_core;
extern crate iron;
extern crate router;
extern crate serde;
extern crate time;

#[macro_use]
extern crate serde_json;

mod middleware;
use self::middleware::Logger;
use iron::error::HttpResult;
use iron::mime::Mime;
use iron::prelude::*;
use iron::status::Status;
use iron::AroundMiddleware;
use iron::Listening;
use router::Router;
use std::net::ToSocketAddrs;

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

include!(concat!(env!("OUT_DIR"), "/version_routes.rs"));

pub fn start_server<A>(addr: A) -> HttpResult<Listening>
where
    A: ToSocketAddrs,
{
    info!("start server");
    let mut router = Router::new();
    add_version_routes(&mut router);
    let iron = Iron::new(Logger::new().around(Box::new(router)));
    iron.http(addr)
}
