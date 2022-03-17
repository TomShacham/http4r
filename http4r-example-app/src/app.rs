use http4r_core::handler::Handler;
use http4r_core::http_message::{Request, Response};
use crate::environment::Environment;
use crate::not_found_handler::NotFoundHandler;
use crate::static_file_handler::StaticFileHandler;

pub struct App<H> where H: Handler {
    handler: H,
    pub env: Environment,
}

impl<H> App<H> where H: Handler {
    pub fn new(handler: H, env: Environment) -> App<H> where H: Handler {
        App {
            handler,
            env,
        }
    }
}

impl<H> Handler for App<H> where H: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        self.handler.handle(req, |res| {
            fun(res)
        })
    }
}

impl<'a> App<NotFoundHandler<StaticFileHandler<'a>>> {
    pub fn in_memory(env: Environment) -> App<NotFoundHandler<StaticFileHandler<'a>>> {
        let env_name = env.get("ENV").unwrap_or("test".to_string());
        App::new(NotFoundHandler::new(StaticFileHandler::new("/resources/html", env_name)), env)
    }

    pub fn production(env: Environment) -> App<NotFoundHandler<StaticFileHandler<'a>>> {
        let env_name = env.get("ENV").unwrap_or("production".to_string());
        App::new(
            NotFoundHandler::new(
                StaticFileHandler::new("/resources/html", env_name)), env)
    }
}

