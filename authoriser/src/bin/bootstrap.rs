mod lib;

use std::error::Error;

use lambda_http::{lambda, IntoResponse, Request};
use lambda_runtime::{error::HandlerError, Context};
use simple_logger;

pub fn handler(req: Request, _c: Context) -> Result<impl IntoResponse, HandlerError> {
    Ok(lib::handler(req))
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug)?;
    lambda!(handler);

    Ok(())
}
