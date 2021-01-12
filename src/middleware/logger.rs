use cli_core::style;
use iron::prelude::*;
use iron::{AroundMiddleware, Handler};

pub struct Logger {}

struct LoggerHandler<H: Handler> {
    logger: Logger,
    handler: H,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {}
    }

    fn log(&self, req: &Request, res: Result<&Response, &IronError>, time: u64) {
        println!(
            "{}",
            format!("{} {:?}", style("Request").bold().green(), req)
        );
        match res {
            Ok(res) => println!(
                "{}",
                format!("{} {:?}", style("Response").bold().green(), res)
            ),
            Err(res) => println!(
                "{}",
                format!("{} {:?}", style("Response").bold().red(), res)
            ),
        };
        println!(
            "{}",
            format!("{} {:?}", style("Response-Time").bold().green(), time)
        );
    }
}

impl<H: Handler> Handler for LoggerHandler<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let entry = ::time::precise_time_ns();
        let res = self.handler.handle(req);
        self.logger
            .log(req, res.as_ref(), ::time::precise_time_ns() - entry);
        res
    }
}

impl AroundMiddleware for Logger {
    fn around(self, handler: Box<dyn Handler>) -> Box<dyn Handler> {
        Box::new(LoggerHandler {
            logger: self,
            handler,
        }) as Box<dyn Handler>
    }
}
