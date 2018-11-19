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
mod routes;
use self::middleware::Logger;
use iron::error::HttpResult;
use iron::prelude::*;
use iron::status::Status;
use iron::AroundMiddleware;
use iron::Listening;
use router::Router;
use std::net::ToSocketAddrs;

fn index_handler(_r: &mut Request) -> IronResult<Response> {
    Ok(Response::with((Status::Ok, "ok")))
}

pub fn start_server<A>(addr: A) -> HttpResult<Listening>
where
    A: ToSocketAddrs,
{
    info!("start server");
    let mut router = Router::new();
    router.get("/", index_handler, "index");
    routes::add_version_routes(&mut router);
    routes::add_versions_route(&mut router);
    let iron = Iron::new(Logger::new().around(Box::new(router)));
    iron.http(addr)
}
